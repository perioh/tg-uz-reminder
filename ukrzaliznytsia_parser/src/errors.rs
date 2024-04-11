use std::num::ParseIntError;

#[derive(thiserror::Error, Debug)]
pub enum UzParseError {
    #[error("Error making web request: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Error parsing html: {0}")]
    HtmlParsing(#[from] html_parser::Error),
    #[error("Error spliting train to get train numbers: {0}")]
    GetTrainNumbers(String),
    #[error("Error spliting train to get train name: {0}")]
    GetTrainName(String),
    #[error("Error parsing train number {0}: {1}")]
    ParseTrainNumber(String, ParseIntError),
    #[error("Error spliting train delay: {0}")]
    SplitDelay(String),
    #[error("Error parsing delay hour: {0}")]
    ParseDelayHour(String, ParseIntError),
    #[error("Error parsing delay minute: {0}")]
    ParseDelayMinute(String, ParseIntError),
    #[error("Dom first child (html content) not found: {0}")]
    DomHtmlNotFound(String),
    #[error("Container with delayed trains not found in dom: {0}")]
    DelayedTrainsContainer(String),
}
