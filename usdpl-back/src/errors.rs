pub enum ServerError {
    Tungstenite(tungstenite::error::Error),
    Io(std::io::Error),
}

pub type ServerResult = Result<(), ServerError>;
