mod consts;
mod mydb;
mod tg;
use chrono::prelude::*;
use chrono::Duration;
use chrono_tz::Europe::Kyiv;
use chrono_tz::Tz;
use consts::SLEEP_BEFORE_FETCH_TRAINS;
use database::Database;
use mydb::MyDb;
use pdf_parser::TicketData;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};
use telegram::ChatId;
use tg::build_train_notification_message;
use tg::telegram_worker;
use tracing::{debug, level_filters::LevelFilter, warn};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use ukrzaliznytsia_parser::UzParserClient;

use crate::consts::NOTIFY_BEFORE_TRAIN;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("dotenvy error");
    let _tracing_guard = init_tracing().expect("Error seting up tracing");

    let telegram_key =
        std::env::var("TELEGRAM_BOT_API_KEY").expect("TELEGRAM_BOT_API_KEY env var not set");
    let tg = telegram::TelegramClient::new(telegram_key);
    let db = MyDb::new();
    let uz_parser = UzParserClient::new();

    tokio::spawn({
        let db = db.clone();
        let tg = tg.clone();
        async {
            telegram_worker(db, tg).await;
        }
    });

    // db.insert_ticket_data(
    //     ChatId(144441960),
    //     TicketData {
    //         departure_datetime: chrono_tz::Europe::Kyiv
    //             .with_ymd_and_hms(2024, 4, 10, 15, 49, 0)
    //             .unwrap(),
    //         train_number: "705".to_owned(),
    //     },
    // )
    // .unwrap();

    let mut notify_before_train_desc = NOTIFY_BEFORE_TRAIN;
    notify_before_train_desc.sort_by(|a, b| b.cmp(a));

    let mut notifications: HashMap<ChatId, HashSet<Duration>> = HashMap::new();

    loop {
        let kyiv_time_now = kyiv_time();

        let delayed_trains_response = uz_parser.delayed_trains().await;
        let delayed_trains = match delayed_trains_response {
            Ok(delayed_trains) => delayed_trains.0,
            Err(e) => {
                warn!(%e,"delayed trains");
                continue;
            }
        };

        let user_with_his_train = |user_train: TicketData| {
            let delayed = delayed_trains
                .iter()
                .find(|delayed_train| delayed_train.numbers.0.contains(&user_train.train_number));
            (user_train, delayed)
        };

        for (user, user_trains) in db.users() {
            let only_recent_trains = |user_train: TicketData| {
                let entry = notifications.entry(user.clone()).or_insert(HashSet::new());

                for timespan in notify_before_train_desc.iter() {
                    if entry.contains(timespan) {
                        continue;
                    } else if user_train.departure_datetime <= kyiv_time_now + *timespan
                        && user_train.departure_datetime > kyiv_time_now
                    {
                        entry.insert(*timespan);
                        return Some(user_train);
                    }
                }

                if entry.len() == notify_before_train_desc.len() {
                    debug!(%user,?user_train, "all notifications sent");
                    if let Err(e) = db.remove_user_train(user, user_train) {
                        warn!(%e,"removing user from db after notifications")
                    }
                }

                None
            };

            let user_delayed_trains = user_trains
                .filter_map(only_recent_trains)
                .map(user_with_his_train);

            for (user_ticket, delayed_train) in user_delayed_trains {
                let message = build_train_notification_message(user_ticket, delayed_train);
                if let Err(e) = tg.send_to_user(user, message).await {
                    warn!(%e,"Error sending message to user telegram");
                }
            }
        }
        tokio::time::sleep(SLEEP_BEFORE_FETCH_TRAINS).await;
    }
}

fn kyiv_time() -> DateTime<Tz> {
    let local_time = chrono::offset::Local::now();

    //unfailable
    Kyiv.with_ymd_and_hms(
        local_time.year(),
        local_time.month(),
        local_time.day(),
        local_time.hour(),
        local_time.minute(),
        local_time.second(),
    )
    .unwrap()
}

fn init_tracing() -> Result<WorkerGuard, Box<dyn Error>> {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("uzbot.log")
        .build("logs")?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let envfilter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into()) //.en
        .from_env()?
        .add_directive("hyper=info".parse()?)
        .add_directive("reqwest=info".parse()?)
        .add_directive("tokio_util=info".parse()?)
        .add_directive("teloxide=off".parse()?)
        .add_directive("h2=info".parse()?);

    let to_log_file = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(non_blocking);

    let fmt_layer = tracing_subscriber::fmt::Layer::new()
        .pretty()
        .with_thread_ids(true)
        .with_line_number(true)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(envfilter)
        .with(to_log_file)
        .init();

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use chrono_tz::Europe::Kyiv;
    use database::Database;
    use pdf_parser::TicketData;
    use telegram::ChatId;

    use crate::mydb::MyDb;
    use chrono::prelude::*;

    #[test]
    fn test_retrieve_user_trains() {
        let db = MyDb::new();

        let departure_datetime = Kyiv.with_ymd_and_hms(2024, 4, 9, 20, 36, 0).unwrap();
        let train_number = "35".to_owned();
        let chat_id1 = ChatId(144441960);
        db.insert_ticket_data(
            chat_id1,
            TicketData {
                departure_datetime,
                train_number: train_number.clone(),
            },
        )
        .unwrap();
        let chat_id2 = ChatId(144441961);

        db.insert_ticket_data(
            chat_id2,
            TicketData {
                departure_datetime,
                train_number: train_number.clone(),
            },
        )
        .unwrap();

        let mut user_trains = db.retrieve_user_trains(chat_id1).collect::<Vec<_>>();

        assert_eq!(user_trains.len(), 1, "len of data must be 1");

        let first_elem = user_trains.remove(0);

        assert_eq!(first_elem.departure_datetime, departure_datetime);

        assert_eq!(first_elem.train_number, train_number);
    }
    #[test]
    fn test_get_all_users() {
        let db = MyDb::new();

        let departure_datetime = Kyiv.with_ymd_and_hms(2024, 4, 9, 20, 36, 0).unwrap();
        let train_number = "35".to_owned();
        let chat_id1 = ChatId(144441960);
        db.insert_ticket_data(
            chat_id1,
            TicketData {
                departure_datetime,
                train_number: train_number.clone(),
            },
        )
        .unwrap();
        let chat_id2 = ChatId(144441961);

        db.insert_ticket_data(
            chat_id2,
            TicketData {
                departure_datetime,
                train_number: train_number.clone(),
            },
        )
        .unwrap();

        let mut user_trains = db.users().collect::<Vec<_>>();

        assert_eq!(
            user_trains.len(),
            2,
            "len of users must be 2, but it is {}",
            user_trains.len()
        );

        let (first_user, first_user_trains) = user_trains.remove(0);

        assert_eq!(first_user, chat_id1, "first user is not {}", chat_id1);
        let mut first_user_trains = first_user_trains.collect::<Vec<_>>();
        assert_eq!(
            first_user_trains.len(),
            1,
            "len of data must be 1, but it is {}",
            first_user_trains.len()
        );

        let first_train = first_user_trains.remove(0);

        assert_eq!(first_train.departure_datetime, departure_datetime);

        assert_eq!(first_train.train_number, train_number);
    }
}
