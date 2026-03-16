use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Site {
    #[serde(default = "default_site_name")]
    pub name:                String,
    pub enable_registration: bool,
}

impl Default for Site {
    fn default() -> Self {
        Self {
            name:                default_site_name(),
            enable_registration: true,
        }
    }
}

fn default_site_name() -> String {
    "Madoka Auth".into()
}
