use chrono::TimeDelta;
use tokio::time::Duration;

pub const SLEEP_BEFORE_FETCH_TRAINS: Duration = Duration::from_secs(10);
pub const NOTIFY_BEFORE_TRAIN: [TimeDelta; 3] = [
    TimeDelta::minutes(15),
    TimeDelta::minutes(30),
    TimeDelta::minutes(60),
];

// pub const NOTIFY_BEFORE_TRAIN: [TimeDelta; 3] = [
//     TimeDelta::seconds(60),
//     TimeDelta::seconds(300),
//     TimeDelta::seconds(150),
// ];

// pub const NOTIFY_BEFORE_TRAIN: [TimeDelta; 3] = [
//     TimeDelta::seconds(15),
//     TimeDelta::seconds(30),
//     TimeDelta::seconds(60),
// ];

pub const EXTRACT_TICKET_ERROR_MESSAGE: &str = "Error extracting pdf data.";
pub const LAYOUT_CHANGED_MESSAGE: &str = "Possibly, ticket layout has changed.";
