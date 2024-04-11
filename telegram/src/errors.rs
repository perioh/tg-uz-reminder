#[derive(Debug, thiserror::Error)]
pub enum TelegramErrors {
    #[error("No document is attached")]
    NoDocumentAttached,
    #[error("No document is attached")]
    TeloxideRequest(#[from] teloxide::errors::RequestError),
}
