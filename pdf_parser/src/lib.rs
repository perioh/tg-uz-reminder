mod errors;

use chrono::{DateTime, TimeZone};
use chrono_tz::{Europe::Kyiv, Tz};
use pdf_extract::OutputError;

use chrono_tz;
pub use errors::ParsePdfError;

pub trait DataExtractor {
    fn extract_text(&self) -> Result<String, OutputError>;
}

impl DataExtractor for &[u8] {
    fn extract_text(&self) -> Result<String, OutputError> {
        pdf_extract::extract_text_from_mem(self)
    }
}

impl DataExtractor for &std::path::Path {
    fn extract_text(&self) -> Result<String, OutputError> {
        pdf_extract::extract_text(self)
    }
}

pub fn parse_departure_data_from_pdf(
    data: impl DataExtractor,
) -> Result<TicketData, ParsePdfError> {
    let string_data = data.extract_text()?;
    let departure_time_full_str = string_data
        .split('\n')
        .find(|line| line.starts_with("Дата/час відпр. "))
        .map_or(Err(ParsePdfError::DepartureDateTimeAbsent), |data| Ok(data))?;

    let mut departure_time_iterator = departure_time_full_str.split_whitespace().skip(2);

    let departure_date = departure_time_iterator
        .next()
        .map_or(Err(ParsePdfError::DepartureDateAbsent), |data| Ok(data))?;
    let departure_time = departure_time_iterator
        .next()
        .map_or(Err(ParsePdfError::DepartureTimeAbsent), |data| Ok(data))?;

    let time_str = format!("{departure_date} {departure_time}");
    let naive = chrono::NaiveDateTime::parse_from_str(&time_str, "%d.%m.%Y %H:%M")?;

    //it must not fail, and I can't use another way to get inner value :-\
    let departure_datetime = Kyiv.from_local_datetime(&naive).unwrap();

    let train_number_line = string_data
        .split('\n')
        .find(|line| line.starts_with("Прізвище, Ім’я"))
        .map_or(Err(ParsePdfError::TrainNumberLineAbsent), |data| Ok(data))?;

    let (_, train_num_line_misc) = train_number_line
        .split_once("Поїзд ")
        .ok_or(ParsePdfError::TrainNumberLineNotPoizd)?;

    let (train_number, _) = train_num_line_misc
        .split_once(' ')
        .ok_or(ParsePdfError::TrainNumberLineNotTrainNumber)?;

    Ok(TicketData {
        departure_datetime,
        train_number: train_number.to_owned(),
    })
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TicketData {
    pub departure_datetime: DateTime<Tz>,
    pub train_number: String,
}
