#[derive(Debug)]
pub enum ServerError {
    Tungstenite(tungstenite::error::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tungstenite(t) => (&t as &dyn std::fmt::Display).fmt(f),
            Self::Io(io) => (&io as &dyn std::fmt::Display).fmt(f),
        }
    }
}

pub type ServerResult = Result<(), ServerError>;
