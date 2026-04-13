use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailSecure {
    #[serde(alias = "tls", alias = "starttls", alias = "STARTTLS")]
    TLS,
    #[serde(alias = "ssl", alias = "smtps", alias = "implicit_tls")]
    SSL,
    #[serde(alias = "none", alias = "plain")]
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mail {
    pub host:     String,
    pub port:     Option<u16>,
    pub username: String,
    pub password: String,
    #[serde(default = "default_secure")]
    pub secure:   MailSecure,
}

fn default_secure() -> MailSecure {
    MailSecure::TLS
}

impl MailSecure {
    pub fn default_port(self) -> u16 {
        match self {
            MailSecure::TLS => 587,
            MailSecure::SSL => 465,
            MailSecure::None => 25,
        }
    }
}

impl Mail {
    pub fn port(&self) -> u16 {
        self.port.unwrap_or_else(|| self.secure.default_port())
    }
}
