#[derive(Debug, thiserror::Error)]
pub enum ParsePdfError {
    #[error("Error parsing pdf: {0}")]
    PdfExtractError(#[from] pdf_extract::OutputError),
    #[error("Departure datetime absent")]
    DepartureDateTimeAbsent,
    #[error("Line with train number absent")]
    TrainNumberLineAbsent,
    #[error("Line with train number doesn't contain Поїзд word")]
    TrainNumberLineNotPoizd,
    #[error("Line with train number doesn't contain train number")]
    TrainNumberLineNotTrainNumber,
    #[error("Departure date absent")]
    DepartureDateAbsent,
    #[error("Departure time absent")]
    DepartureTimeAbsent,
    #[error("Erorr parsing time: {0}")]
    TimeParse(#[from] chrono::ParseError),
    #[error("Erorr parsing train number: {0}")]
    ParseTrainNumber(String),
}
