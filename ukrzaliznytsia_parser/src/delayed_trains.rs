use crate::errors::UzParseError;
use std::{fmt::Display, str::FromStr};

#[derive(Debug)]
pub struct DelayedTrains(pub Vec<DelayedTrain>);

#[derive(Debug)]
pub struct DelayedTrain {
    pub direction: TrainDirection,
    pub numbers: TrainNumbers,
    pub delay: TrainDelayTime,
}

#[derive(Debug)]
pub struct TrainNumbers(pub Vec<String>);

#[derive(Debug)]
pub struct TrainDelayTime {
    pub hr: usize,
    pub min: usize,
}
#[derive(Debug)]
pub struct TrainDirection(String);

impl Display for TrainDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DelayedTrains {
    type Err = UzParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let required_class: String = String::from("delayform-list");

        let dom = html_parser::Dom::parse(input)?;
        let html = dom
            .children
            .first()
            .ok_or(UzParseError::DomHtmlNotFound(input.to_owned()))?;

        let delayed_container = html
            .into_iter()
            .filter_map(|node| node.element())
            .find(|element| element.classes.contains(&required_class))
            .ok_or(UzParseError::DelayedTrainsContainer(input.to_owned()))?;

        let delayed_trains_iterator = delayed_container
            .children
            .iter()
            .filter_map(|node| node.element())
            .filter_map(|ch| ch.children.first())
            .filter_map(|child| child.text());

        let mut delayed_trains = vec![];
        for train in delayed_trains_iterator {
            let (numbers, rest) = train
                .split_once(' ')
                .ok_or(UzParseError::GetTrainNumbers(train.to_owned()))?;
            let (train_direction, rest) = rest
                .split_once(" (+")
                .ok_or(UzParseError::GetTrainName(train.to_owned()))?;
            let train_delay = &rest[..rest.len() - 1];

            let train_numbers: TrainNumbers = numbers.parse()?;
            let train_direction = TrainDirection(train_direction.to_owned());
            let train_delay: TrainDelayTime = train_delay.parse()?;

            delayed_trains.push(DelayedTrain {
                direction: train_direction,
                numbers: train_numbers,
                delay: train_delay,
            })
        }

        Ok(DelayedTrains(delayed_trains))
    }
}

impl FromStr for TrainNumbers {
    type Err = UzParseError;

    fn from_str(numbers: &str) -> Result<Self, Self::Err> {
        // skip 1 because 1st element is `№` sign
        let numbers = numbers.chars().skip(1).collect::<String>();

        let numbers = numbers
            .split("/")
            .map(str::to_owned)
            .collect::<Vec<String>>();
        Ok(TrainNumbers(numbers))
    }
}

impl FromStr for TrainDelayTime {
    type Err = UzParseError;

    fn from_str(delay: &str) -> Result<Self, Self::Err> {
        let (hr, min) = delay
            .split_once(":")
            .ok_or(UzParseError::SplitDelay(delay.to_owned()))?;

        let hr: usize = hr
            .parse()
            .map_err(|e| UzParseError::ParseDelayHour(hr.to_owned(), e))?;
        let min: usize = min
            .parse()
            .map_err(|e| UzParseError::ParseDelayHour(min.to_owned(), e))?;

        Ok(TrainDelayTime { hr, min })
    }
}
