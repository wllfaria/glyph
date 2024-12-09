#[derive(Debug)]
pub enum MappableCommand {
    Static { name: String },
    Dynamic { callback: fn() },
}

impl<S> From<S> for MappableCommand
where
    S: AsRef<str>,
{
    fn from(value: S) -> MappableCommand {
        MappableCommand::Static {
            name: value.as_ref().to_string(),
        }
    }
}
