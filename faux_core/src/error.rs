pub type Result<T> = std::result::Result<T, FauxError>;

#[derive(Debug, thiserror::Error)]
pub enum FauxError {
    #[error("failed to decode faux packet")]
    DecodeError(std::io::Error),
    #[error("failed to start server")]
    ServerStartError(std::io::Error),
    #[error("server failed unexpectedly")]
    ServerError(std::io::Error),
}
