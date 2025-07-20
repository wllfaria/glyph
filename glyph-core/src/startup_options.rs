use clap::Parser;

#[derive(Debug, Parser, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[command(version, about, long_about = None)]
pub struct StartupOptions {
    #[arg()]
    pub files: Vec<String>,
}

impl StartupOptions {
    pub fn from_args() -> Self {
        Self::parse()
    }
}
