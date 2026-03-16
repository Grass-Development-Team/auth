use std::{path::Path, sync::Arc};

use anyhow::{Context, Result, anyhow};
use assets::AssetManager;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use minijinja::{AutoEscape, Environment, Value};

use crate::internal::config::{Mail, MailSecure};

pub fn init(config: &Mail) -> Result<Mailer> {
    Mailer::new(config)
}

#[derive(Clone)]
pub struct Mailer {
    sender:    Mailbox,
    transport: SmtpTransport,
    templates: Arc<Environment<'static>>,
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

        let templates = Self::load_templates()?;

        Ok(Self {
            sender,
            transport,
            templates: Arc::new(templates),
        })
    }

    pub async fn send_mail(
        &self,
        to: &str,
        subject: &str,
        template: &str,
        context: Value,
    ) -> Result<()> {
        let content = self
            .templates
            .get_template(template)
            .with_context(|| format!("Mail template not found: {template}"))?
            .render(context)
            .with_context(|| format!("Failed to render mail template: {template}"))?;

        self.send_html(to, subject, &content).await
    }

    async fn send_html(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let to: Mailbox = to
            .parse()
            .with_context(|| format!("Invalid recipient address: {to}"))?;

        let message = Message::builder()
            .from(self.sender.clone())
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_owned())
            .context("Failed to build email message")?;

        let transport = self.transport.clone();
        tokio::task::spawn_blocking(move || transport.send(&message))
            .await
            .context("Email sending task failed")?
            .context("SMTP send failed")?;

        Ok(())
    }

    fn load_templates() -> Result<Environment<'static>> {
        let mut env = Environment::new();
        env.set_auto_escape_callback(|_| AutoEscape::Html);

        let mut loaded = 0usize;
        for (path, file) in AssetManager::get_dir("templates/mails") {
            if !path.ends_with(".html") {
                continue;
            }

            let Some(filename) = Path::new(&path).file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            let Some(name) = filename.strip_suffix(".html") else {
                continue;
            };

            let source = String::from_utf8(file.data.into_owned())
                .with_context(|| format!("Mail template is not valid UTF-8: {path}"))?;
            let name: &'static str = Box::leak(name.to_owned().into_boxed_str());
            let source: &'static str = Box::leak(source.into_boxed_str());

            env.add_template(name, source)
                .with_context(|| format!("Failed to parse mail template: {path}"))?;
            loaded += 1;
        }

        if loaded == 0 {
            return Err(anyhow!(
                "No mail templates found under templates/mails/*.html"
            ));
        }

        Ok(env)
    }
}
