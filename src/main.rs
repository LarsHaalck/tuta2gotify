use anyhow::Result;
use gotify::AppClient as GotifyClient;
use handlebars::Handlebars;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, warn};
use tuta_poll::client::{Client, MailContent};

mod config;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "c", long = "config")]
    config_file: Option<PathBuf>,
}

fn format(handlebars: &Handlebars, mail: MailContent) -> Result<String> {
    Ok(handlebars.render(
        "format",
        &serde_json::json!({
            "name": mail.name.unwrap_or("?".into()),
            "address": mail.address,
            "subject": mail.subject.unwrap_or("?".into()),
            "body": html2text::from_read(mail.body.unwrap_or("?".into()).as_bytes(), 90),
        }),
    )?)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let options = Options::from_args();
    let config_file = match options.config_file {
        Some(file) => file,
        None => dirs::config_local_dir()
            .expect("Config dir does not exist")
            .join("tuta2gotfy")
            .join("config.toml"),
    };
    let config = config::Config::read(config_file)?;
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("format", &config.gotify.format)?;
    let client = Client::new(&config.account).await?;
    let gotify_client: GotifyClient =
        gotify::Client::new(config.gotify.url.as_str(), &config.gotify.token)?;

    let mails = client.get_mails().await?;
    let num_mails = mails.len();
    let mut unread_mails: Vec<_> = mails.into_iter().filter(|m| m.unread == "1").collect();
    info!("Got {} mails, {} unread", num_mails, unread_mails.len());
    for mail in &mut unread_mails {
        if mail.unread == "0" {
            continue;
        }
        let decrypted_mail = client.decrypt(&mail).await?;
        let _ = gotify_client
            .create_message(format(&handlebars, decrypted_mail)?)
            .await;
        client.mark_read(mail).await?;
    }

    let webscoket_connector = client.get_websocket_connector()?;

    loop {
        let mut socket = webscoket_connector.connect()?;
        while let Ok(mut mails) = socket.read_create().await {
            for mail in &mut mails {
                let decrypted_mail = client.decrypt(mail).await?;
                let _ = gotify_client
                    .create_message(format(&handlebars, decrypted_mail)?)
                    .send()
                    .await;
                client.mark_read(mail).await?;
            }
        }
        warn!("Error");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
