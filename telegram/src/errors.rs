#[derive(Debug, thiserror::Error)]
pub enum TelegramErrors {
    #[error("No document is attached")]
    NoDocumentAttached,
    #[error("Teloxide request error: {0}")]
    TeloxideRequest(#[from] teloxide::errors::RequestError),
}
