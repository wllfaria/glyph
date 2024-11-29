use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::BaseDirs;

pub static DIRS: OnceLock<Dirs> = OnceLock::new();

#[derive(Debug)]
pub struct Dirs {
    base_dirs: BaseDirs,
    config_dir: PathBuf,
    config: PathBuf,
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

        Dirs {
            config,
            config_dir,
            base_dirs,
        }
    }

    pub fn base_dirs(&self) -> &BaseDirs {
        &self.base_dirs
    }

    pub fn config(&self) -> &Path {
        &self.config
    }
}

impl Default for Dirs {
    fn default() -> Dirs {
        Dirs::new()
    }
}

pub fn setup_dirs() {
    DIRS.get_or_init(Default::default);
}
