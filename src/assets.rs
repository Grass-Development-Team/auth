use std::path::{Component, Path, PathBuf};

use rust_embed::{Embed, EmbeddedFile};

#[derive(Embed)]
#[folder = "assets/"]
#[include = "public/**/*"]
#[include = "templates/**/*"]
#[exclude = "*.DS_Store"]
pub struct Assets;

#[derive(Clone, Copy)]
pub struct AssetManager;

impl AssetManager {
    pub fn get(path: &str) -> Option<EmbeddedFile> {
        let normalized = path.trim_start_matches('/');
        if !Self::is_safe_relative_path(normalized) {
            return None;
        }

        if let Some(file) = Self::read_local(normalized) {
            return Some(file);
        }

        Assets::get(&Self::to_embed_path(normalized))
    }

    fn read_local(path: &str) -> Option<EmbeddedFile> {
        let local_path = Self::to_local_path(path)?;
        rust_embed::utils::read_file_from_fs(&local_path).ok()
    }

    fn to_local_path(path: &str) -> Option<PathBuf> {
        if let Some(relative) = path.strip_prefix("public/") {
            let base = Path::new("public");
            if base.is_dir() {
                return Some(base.join(relative));
            }
            return None;
        }

        if let Some(relative) = path.strip_prefix("template/") {
            let base = Path::new("template");
            if base.is_dir() {
                return Some(base.join(relative));
            }
            return None;
        }

        None
    }

    fn to_embed_path(path: &str) -> String {
        if let Some(relative) = path.strip_prefix("template/") {
            return format!("templates/{relative}");
        }

        path.to_owned()
    }

    fn is_safe_relative_path(path: &str) -> bool {
        if path.is_empty() {
            return false;
        }

        Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    }
}
