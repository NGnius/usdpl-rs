#[derive(Debug)]
pub enum ServerError {
    Io(std::io::Error),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => (&io as &dyn std::fmt::Display).fmt(f),
        }
    }
}

pub type ServerResult = Result<(), ServerError>;
