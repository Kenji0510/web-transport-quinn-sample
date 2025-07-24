use anyhow::Context;
use clap::Parser;
use std::{fs, io, path};
use url::Url;
use web_transport_quinn::quinn::rustls::pki_types::CertificateDer;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "https://localhost:4443")]
    url: Url,

    #[arg(long, default_value = "cert/localhost.crt")]
    pub tls_cert: path::PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

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

    // let (mut send, mut recv) = session.open_bi().await?;

    log::info!("Created stream");

    for i in 0..5 {
        let (mut send, mut recv) = session.open_bi().await?;

        log::info!("Sending message {}", i + 1);
        let msg = "Hello!, world!".to_string();
        send.write_all(msg.as_bytes()).await?;
        log::info!("Sent: {}", msg);

        send.finish()?;

        let msg = recv.read_to_end(1024).await?;
        log::info!("Recv: {}", String::from_utf8_lossy(&msg));

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
