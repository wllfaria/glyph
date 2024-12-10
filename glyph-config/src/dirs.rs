use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::BaseDirs;

pub static DIRS: OnceLock<Dirs> = OnceLock::new();

#[derive(Debug)]
pub struct Dirs {
    base_dirs: BaseDirs,
    config: PathBuf,
    data: PathBuf,
}

impl Dirs {
    pub fn new() -> Dirs {
        let base_dirs = BaseDirs::new().expect("failed to initialize base directories");

        #[cfg(target_os = "macos")]
        let config_dir = {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home).join(".config")
            } else {
                base_dirs.config_dir().to_path_buf()
            }
        };

        #[cfg(not(target_os = "macos"))]
        let config_dir = base_dirs.config_dir();

        let config = config_dir.join("glyph");

        #[cfg(target_os = "macos")]
        let data_dir = 'block: {
            if let Ok(home) = std::env::var("HOME") {
                let local_share = PathBuf::from(home).join(".local").join("share");
                if std::fs::exists(&local_share).unwrap_or_default() {
                    break 'block local_share;
                }
            }
            base_dirs.data_dir().to_path_buf()
        };

        #[cfg(not(target_os = "macos"))]
        let data_dir = base_dirs.data_dir();

        let data = data_dir.join("glyph");

        if !std::fs::exists(&data).unwrap_or_default() {
            std::fs::create_dir(&data).expect("failed to initialize data directory");
        }

        Dirs {
            config,
            base_dirs,
            data,
        }
    }

    pub fn base_dirs(&self) -> &BaseDirs {
        &self.base_dirs
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn data(&self) -> &Path {
        &self.data
    }
}

impl Default for Dirs {
    fn default() -> Dirs {
        Dirs::new()
    }
}

pub fn setup_dirs() -> &'static Dirs {
    DIRS.get_or_init(Default::default)
}
