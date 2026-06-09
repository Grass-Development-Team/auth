use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    #[default]
    Redis,
    Moka,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MokaCacheConfig {
    pub max_capacity: u64,
}

impl Default for MokaCacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CacheConfig {
    /// Backend type: "redis" for multi-instance, or "moka" for single-instance
    /// only.
    #[serde(default)]
    pub backend: CacheBackend,
    /// Moka backend configuration. Only used when backend is "moka".
    #[serde(default)]
    pub moka:    MokaCacheConfig,
}
