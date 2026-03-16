use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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
        let normalized = Self::normalize(path, false)?;

        if let Some(local_path) = Self::resolve_local_path(normalized)
            && let Ok(file) = rust_embed::utils::read_file_from_fs(&local_path)
        {
            return Some(file);
        }

        Assets::get(normalized)
    }

    pub fn get_dir(path: &str) -> Vec<(String, EmbeddedFile)> {
        let Some(normalized) = Self::normalize(path, true) else {
            return Vec::new();
        };

        if let Some(local_root) = Self::resolve_local_path(normalized)
            && local_root.is_dir()
        {
            let local_files = Self::collect_local_files(normalized, &local_root);
            if !local_files.is_empty() {
                return local_files;
            }
        }

        Self::collect_embedded_files(normalized)
    }

    fn collect_local_files(path: &str, local_root: &Path) -> Vec<(String, EmbeddedFile)> {
        let mut files = Vec::new();
        let mut stack = vec![local_root.to_path_buf()];

        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                    continue;
                }

                let Ok(relative) = entry_path.strip_prefix(local_root) else {
                    continue;
                };
                let relative = relative
                    .components()
                    .filter_map(|component| match component {
                        Component::Normal(v) => Some(v.to_string_lossy().into_owned()),
                        _ => None,
                    })
                    .collect::<Vec<String>>()
                    .join("/");

                let logical_path = format!("{path}/{relative}");
                if let Ok(file) = rust_embed::utils::read_file_from_fs(&entry_path) {
                    files.push((logical_path, file));
                }
            }
        }

        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    }

    fn collect_embedded_files(path: &str) -> Vec<(String, EmbeddedFile)> {
        let embed_prefix = format!("{path}/");
        let mut files = Vec::new();

        for file_path in Assets::iter() {
            let file_path = file_path.as_ref();
            if !file_path.starts_with(&embed_prefix) {
                continue;
            }

            if let Some(file) = Assets::get(file_path) {
                files.push((file_path.to_owned(), file));
            }
        }

        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    }

    fn resolve_local_path(path: &str) -> Option<PathBuf> {
        if let Some(relative) = path.strip_prefix("public/") {
            let base = Path::new("public");
            if base.is_dir() {
                return Some(base.join(relative));
            }
            return None;
        }

        if let Some(relative) = path.strip_prefix("templates/") {
            let base = Path::new("templates");
            if base.is_dir() {
                return Some(base.join(relative));
            }
            return None;
        }

        None
    }

    fn normalize(path: &str, trim_end_slash: bool) -> Option<&str> {
        let path = path.trim_start_matches('/');
        let path = if trim_end_slash {
            path.trim_end_matches('/')
        } else {
            path
        };

        if Self::is_safe_path(path) {
            Some(path)
        } else {
            None
        }
    }

    fn is_safe_path(path: &str) -> bool {
        if path.is_empty() {
            return false;
        }

        Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    }
}
