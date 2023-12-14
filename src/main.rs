use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::{Error, Result};
use tracing::{debug, info};
use tuta_poll::client::Client;

mod config;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "c", long = "config")]
    config_file: Option<PathBuf>,
}

fn main() -> Result<()> {
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
    let client = Client::new(&config.account)?;

    let mails = client.get_mails()?;
    let unread_mails : Vec<_> = mails.iter().filter(|m| m.unread == "1").collect();
    info!("Got {} mails, {} unread", mails.len(), unread_mails.len());
    for mail in unread_mails {
        if mail.unread == "0" {
            continue;
        }
        let decrypted_mail = client.decrypt(&mail);
        debug!("Got mail: {:?}", decrypted_mail);

        // client.mark_read(&mut mail)?;
    }
    Ok(())

    // loop {
    //     let mut socket = client.websocket();

    //     while let Ok(mails) = socket.read() {
    //         for mail in mails {
    //             let decrypted_mail = client.decrypt(mail);
    //         }

    //     }
    //     println!("Error");
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    // }
}
