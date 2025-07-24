use anyhow::Context;
use clap::Parser;
use std::{fs, io, path};

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
    loop {
        tokio::select! {
            res = session.accept_bi() => {
                match res {
                    Ok((send, recv)) => {
                        log::info!("Accepted stream");

                        tokio::spawn(async move {
                            if let Err(e) = handle_bi_stream(send, recv).await {
                                log::error!("Failed to handle stream: {}", e);
                            }
                        });
                    }
                    Err(err) => {
                        log::error!("Failed to accept stream: {}", err);
                        return Err(err.into());
                    }
                }
            },
            res = session.read_datagram() => {
                let msg = res?;
                log::info!("Accepted datagram");
                log::info!("Recv: {}", String::from_utf8_lossy(&msg));

                session.send_datagram(msg.clone())?;
                log::info!("Send: {}", String::from_utf8_lossy(&msg));
            },
        };

        log::info!("Echo successful!");
    }
}

async fn handle_bi_stream(
    mut send: web_transport_quinn::SendStream,
    mut recv: web_transport_quinn::RecvStream,
) -> anyhow::Result<()> {
    log::info!("Handling bidirectional stream");

    let msg = recv.read_to_end(1024).await?;
    log::info!("Stream received: {}", String::from_utf8_lossy(&msg));

    send.write_all(&msg).await?;
    send.finish()?;

    log::info!("Stream sent: {}", String::from_utf8_lossy(&msg));
    Ok(())
}
