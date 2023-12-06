use super::config;
use anyhow::{Error, Result};
use lz4_flex::decompress_into;
use tuta_poll::mail::Mail;
use tuta_poll::mailfolder::{Folder, MailFolderType};
use tuta_poll::user::GroupType;
use tuta_poll::*;

pub struct Client {
    config: config::Account,
    access_token: String,
    mail_group_key: [u8; 16],
    inboxes: Vec<Folder>,
}

#[derive(Debug)]
pub struct MailContent {
    pub subject: Option<String>,
    pub name: Option<String>,
    pub address: String,
    pub body: Option<String>,
}

impl Client {
    pub fn new(config: &config::Account) -> Result<Client> {
        let salt = salt::fetch(&config.email_address)?;
        let user_passphrase_key = crypto::create_user_passphrase_key(&config.password, &salt);
        let session = session::fetch(&config.email_address, &user_passphrase_key)?;

        let access_token = session.access_token;
        let user = user::fetch(&access_token, &session.user)?;

        let mail_member = user
            .memberships
            .iter()
            .find(|membership| membership.group_type == GroupType::Mail)
            .ok_or(Error::msg("Could not find group with type mail"))?;

        let user_group_key =
            crypto::decrypt_key(&user_passphrase_key, &user.user_group.sym_enc_g_key)?;
        let mail_group_key = crypto::decrypt_key(&user_group_key, &mail_member.sym_enc_g_key)?;
        let root = mailboxgrouproot::fetch(&access_token, &mail_member.group)?;

        let mailbox = mailbox::fetch(&access_token, &root)?;
        let folders = mailfolder::fetch(&access_token, &mailbox)?;

        let inboxes: Vec<_> = folders
            .into_iter()
            .filter(|folder| folder.folder_type == MailFolderType::Inbox)
            .collect();
        Ok(Client {
            config: config.clone(),
            access_token,
            mail_group_key,
            inboxes,
        })
    }

    pub fn get_mails(&self) -> Result<Vec<Mail>> {
        let mut mails = Vec::new();
        for inbox in &self.inboxes {
            mails.extend(mail::fetch_from_inbox(&self.access_token, &inbox.mails)?);
        }
        Ok(mails)
    }

    // new owner_enc_session_key
    // let owner_enc_session_key = crypto::decrypt_key(
    //     &self.mail_group_key,
    //     &mail.owner_enc_session_key.as_ref().unwrap(),
    // )

    pub fn decrypt(&self, mail: &Mail) -> Result<MailContent> {
        // owner_enc_session_key should also be Some
        let session_key = crypto::decrypt_key(
            &self.mail_group_key,
            &mail.owner_enc_session_key.as_ref().unwrap(),
        )
        .expect("Could not retrieve session key");
        let session_sub_keys = crypto::SubKeys::new(session_key);

        let subject = if self.config.show_subject {
            let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.subject)?;
            Some(
                std::str::from_utf8(&tmp)
                    .expect("Subject could not converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        let name = if self.config.show_name {
            let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.sender.name)?;
            Some(
                std::str::from_utf8(&tmp)
                    .expect("Name could not converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        let address = mail.sender.address.to_string();

        let body = if self.config.show_body {
            let mailbody = mailbody::fetch(&self.access_token, &mail.body)?;
            let compressed_text = crypto::decrypt_with_mac(&session_sub_keys, &mailbody)?;
            let mut buf: Vec<u8> = vec![0; mailbody.len() * 6];
            let size = decompress_into(&compressed_text, &mut buf)?;
            buf.resize(size, 0);
            Some(
                std::str::from_utf8(&buf)
                    .expect("Body could not be converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        Ok(MailContent {
            subject,
            name,
            address,
            body,
        })
    }

    pub fn mark_read(&self, mail: &mut Mail) -> Result<()> {
        if mail.unread == "0" {
            return Ok(());
        }

        mail.unread = "0".to_string();
        mail::update(&self.access_token, &mail)?;
        Ok(())
    }
}
