use anyhow::Result;
use futures_util::pin_mut;
use futures_util::StreamExt;
use gotify::AppClient as GotifyClient;
use handlebars::Handlebars;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, warn, debug};
use tuta_poll::client::{Client, MailContent};
use tuta_poll::types::ReadStatus;

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

async fn relay_mails(
    client: &Client,
    gotify_client: &GotifyClient,
    handlebars: &Handlebars<'_>,
) -> Result<()> {
    let mails = client.get_mails();
    pin_mut!(mails);
    while let Some(mail) = mails.next().await {
        let mut mail = mail?;
        if mail.read_status == ReadStatus::Read {
            continue;
        }
        info!("Relaying a new mail by: {:?}", mail.sender.address);
        let decrypted_mail = client.decrypt(&mail).await?;
        let _ = gotify_client
            .create_message(format(&handlebars, decrypted_mail)?)
            .await;
        client.set_read_status(&mut mail, ReadStatus::Read).await?;
    }

    Ok(())
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

    relay_mails(&client, &gotify_client, &handlebars).await?;
    let webscoket_connector = client.get_websocket_connector()?;

    loop {
        let mut socket = webscoket_connector.connect()?;
        while let Ok(has_new) = socket.has_new().await {
            if !has_new {
                continue;
            }
            debug!("Got new mails by websocket event");
            relay_mails(&client, &gotify_client, &handlebars).await?;
        }
        warn!("Socket error. Retrying in 10s");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
