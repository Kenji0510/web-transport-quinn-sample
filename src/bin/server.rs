use anyhow::Context;
use bincode::{Decode, Encode};
use clap::Parser;
use serde::{Deserialize, Serialize};
use web_transport_rs_sample::operate_pcd::{save_pcd, PointXYZ};
use std::{collections::HashMap, fs, io, path};

use web_transport_quinn::{Session, quinn::rustls::pki_types::CertificateDer};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "[::]:4443")]
    addr: std::net::SocketAddr,

    #[arg(long, default_value = "cert/localhost.crt")]
    pub tls_cert: path::PathBuf,

    #[arg(long, default_value = "cert/localhost.key")]
    pub tls_key: path::PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
struct PointChunk {
    chunk_id: u32,
    total_chunks: u32,
    points: Vec<PointXYZ>,
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
struct TransmissionComplete {
    total_points: usize,
    total_chunks: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    let args = Args::parse();

    let chain = fs::File::open(args.tls_cert).context("Failed to open TLS certificate")?;
    let mut chain = io::BufReader::new(chain);

    let chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut chain)
        .collect::<Result<_, _>>()
        .context("Failed to load certs")?;

    anyhow::ensure!(!chain.is_empty(), "Could not find certificate");

    let keys = fs::File::open(args.tls_key).context("Failed to open key file")?;

    let key = rustls_pemfile::private_key(&mut io::BufReader::new(keys))
        .context("Failed to load private key")?
        .context("Missing private key")?;

    let mut server = web_transport_quinn::ServerBuilder::new()
        .with_addr(args.addr)
        .with_certificate(chain, key)?;

    log::info!("listening on {}", args.addr);

    while let Some(conn) = server.accept().await {
        tokio::spawn(async move {
            let err = run_conn(conn).await;
            if let Err(err) = err {
                log::error!("Connection failed: {}", err);
            }
        });
    }

    Ok(())
}

async fn run_conn(request: web_transport_quinn::Request) -> anyhow::Result<()> {
    log::info!("Received WebTransport request: {}", request.url());

    let session = request.ok().await.context("Failed to accept session")?;
    log::info!("Accepted session!");

    if let Err(err) = run_session(session).await {
        log::info!("Closing session: {}", err);
    }

    Ok(())
}

async fn run_session(session: Session) -> anyhow::Result<()> {
    let mut received_chunks: HashMap<u32, PointChunk> = HashMap::new();

    loop {
        tokio::select! {
            res = session.accept_bi() => {
                let (mut send, mut recv) = res?;
                log::debug!("Accepted stream");

                let msg = recv.read_to_end(1024).await?;
                log::debug!("Recv: {}", String::from_utf8_lossy(&msg));

                send.write_all(&msg).await?;
                log::debug!("Send: {}", String::from_utf8_lossy(&msg));
            },
            res = session.read_datagram() => {
                let msg = res?;
                log::debug!("Accepted datagram: {}", msg.len());

                match bincode::decode_from_slice::<PointChunk, _>(&msg, bincode::config::standard()) {
                    Ok((point_chunk, _)) => {
                        log::debug!(
                            "Received chunk {}/{} with {} points",
                            point_chunk.chunk_id + 1,
                            point_chunk.total_chunks,
                            point_chunk.points.len()
                        );

                        received_chunks.insert(point_chunk.chunk_id, point_chunk);
                    },
                    Err(_) => {
                        match bincode::decode_from_slice::<TransmissionComplete, _>(&msg, bincode::config::standard()) {
                            Ok((completion, _)) => {
                                log::info!(
                                    "Transmission complete! Total points: {}, Total chunks: {}",
                                    completion.total_points,
                                    completion.total_chunks
                                );

                                reconstruct_pcd_data(&received_chunks, &completion).await?;
                            },
                            Err(e) => {
                                log::error!("Failed to decode datagram: {}", e);
                            }
                        }
                    }
                }
            },
        };
    }
}

async fn reconstruct_pcd_data(
    chunks: &HashMap<u32, PointChunk>, 
    completion: &TransmissionComplete
) -> anyhow::Result<()> {
    let mut all_points = Vec::new();
    
    // chunk_idでソートして順序を保持
    let mut sorted_chunks: Vec<_> = chunks.iter().collect();
    sorted_chunks.sort_by_key(|(chunk_id, _)| *chunk_id);
    
    for (_, chunk) in sorted_chunks {
        all_points.extend_from_slice(&chunk.points);
    }
    
    log::info!(
        "Reconstructed PCD data: {} total points from {} chunks", 
        all_points.len(),
        chunks.len()
    );

    match save_pcd("data/output/received.pcd", &all_points) {
        Ok(_) => log::info!("Saved received PCD to data/output/received.pcd"),
        Err(e) => log::error!("Failed to save received PCD: {}", e),
    }

    Ok(())
}