use anyhow::{Context, Result};
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::internal::config::{Mail, MailSecure};

pub fn init(config: &Mail) -> Result<Mailer> {
    Mailer::new(config)
}

#[derive(Clone)]
pub struct Mailer {
    sender:    Mailbox,
    transport: SmtpTransport,
}

impl Mailer {
    pub fn new(config: &Mail) -> Result<Self> {
        let sender: Mailbox = config
            .username
            .parse()
            .context("mail.username must be a valid email address")?;

        let builder = match config.secure {
            MailSecure::TLS => SmtpTransport::starttls_relay(&config.host).with_context(|| {
                format!(
                    "Failed to create STARTTLS transport for host {}",
                    config.host
                )
            })?,
            MailSecure::SSL => SmtpTransport::relay(&config.host).with_context(|| {
                format!("Failed to create SMTPS transport for host {}", config.host)
            })?,
            MailSecure::None => SmtpTransport::builder_dangerous(config.host.clone()),
        };

        let transport = builder
            .port(config.port())
            .credentials(Credentials::new(
                config.username.clone(),
                config.password.clone(),
            ))
            .build();

        Ok(Self { sender, transport })
    }

    pub async fn send_plain_text(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let to: Mailbox = to
            .parse()
            .with_context(|| format!("Invalid recipient address: {to}"))?;

        let message = Message::builder()
            .from(self.sender.clone())
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_owned())
            .context("Failed to build email message")?;

        let transport = self.transport.clone();
        tokio::task::spawn_blocking(move || transport.send(&message))
            .await
            .context("Email sending task failed")?
            .context("SMTP send failed")?;

        Ok(())
    }
}
