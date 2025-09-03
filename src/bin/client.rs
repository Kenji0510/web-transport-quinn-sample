use anyhow::Context;
use bincode::{Decode, Encode};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};
use url::Url;
use web_transport_quinn::quinn::rustls::pki_types::CertificateDer;

use web_transport_rs_sample::operate_pcd::{load_pcd, PointXYZ};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "https://localhost:4443")]
    url: Url,

    #[arg(long, default_value = "cert/localhost.crt")]
    pub tls_cert: path::PathBuf,
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

async fn send_pcd_unreliable(
    session: &web_transport_quinn::Session,
    pcd_data: &[PointXYZ],
    chunk_size: usize,
    send_interval_ms: u64,
) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();

    let chunks : Vec<&[PointXYZ]> = pcd_data.chunks(chunk_size).collect();
    let total_chunks = chunks.len() as u32;

    log::info!(
        "Sending {} points in {} chunks via unreliable stream",
        pcd_data.len(),
        total_chunks
    );

    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_data = PointChunk {
            chunk_id: i as u32,
            total_chunks,
            points: chunk.to_vec(),
        };

        let serialized = bincode::encode_to_vec(&chunk_data, bincode::config::standard())?;
        let serialized_size = serialized.len();

        // Send datagram
        session.send_datagram(serialized.into())
            .context("Failed to send datagram")?;

        log::debug!(
            "Sent chunk {}/{} ({} points, {} bytes)",
            i + 1,
            total_chunks,
            chunk.len(),
            serialized_size
        );

        // tokio::time::sleep(tokio::time::Duration::from_millis(send_interval_ms)).await;
        // tokio::time::sleep(tokio::time::Duration::from_micros(send_interval_ms)).await;
    }

    let completion = TransmissionComplete {
        total_points: pcd_data.len(),
        total_chunks: total_chunks as usize,
    };
    let completion_data = bincode::encode_to_vec(&completion, bincode::config::standard())?;
    session.send_datagram(completion_data.into())?;

    let elapsed = start_time.elapsed();

    log::debug!("Sent transmission complete message");
    log::info!("Send time: {:?}", elapsed);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    // Load the pcd file
    let pcd_file_path = "data/input/Sample01.pcd";
    let pcd = match load_pcd(pcd_file_path) {
        Ok(points) => {
            println!("Loaded {} points from {}", points.len(), pcd_file_path);
            points
        }
        Err(e) => {
            eprintln!("Error loading PCD file: {}", e);
            return Err(e);
        }
    };

    // WebTransport
    let args = Args::parse();

    let chain = fs::File::open(args.tls_cert).context("Failed to open cert file")?;
    let mut chain = io::BufReader::new(chain);

    let chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut chain)
        .collect::<Result<_, _>>()
        .context("Failed to load certs")?;

    let client = web_transport_quinn::ClientBuilder::new().with_server_certificates(chain)?;

    log::info!("Connecting to {}", args.url);

    let session = client.connect(args.url).await?;

    log::info!("Connected");

    let chunk_size = 110;

    match send_pcd_unreliable(&session, &pcd, chunk_size, 1).await {
        Ok(_) => {
            log::info!("Successfully sent all PCD data");
        }
        Err(e) => {
            log::error!("Failed to send PCD data: {}", e);
            return Err(e.into());
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    log::info!("Client shutting down");

    Ok(())
}
