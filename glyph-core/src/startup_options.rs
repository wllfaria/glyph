use clap::Parser;

#[derive(Debug, Parser, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[command(version, about, long_about = None)]
pub struct StartupOptions {
    #[arg()]
    pub files: Vec<String>,
    #[arg(short, long)]
    pub config: Option<String>,
}

impl StartupOptions {
    pub fn from_args() -> Self {
        Self::parse()
    }
}
