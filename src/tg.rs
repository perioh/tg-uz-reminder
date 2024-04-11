use std::collections::HashSet;

use chrono::{Duration, Timelike};
use database::Database;
use pdf_parser::TicketData;
use telegram::{ChatId, TelegramClient};
use tracing::{error, trace};
use ukrzaliznytsia_parser::DelayedTrain;

use crate::{
    consts::{EXTRACT_TICKET_ERROR_MESSAGE, LAYOUT_CHANGED_MESSAGE},
    mydb::MyDb,
};

pub async fn telegram_worker(db: MyDb<ChatId, HashSet<TicketData>>, tg: TelegramClient) {
    tg.receive_messages(move |user, file_content| {
        let db = db.clone();
        async move {
            let parsed_pdf_resp = tokio::task::spawn_blocking(move || {
                pdf_parser::parse_departure_data_from_pdf(&*file_content)
            })
            .await;

            let parsed_pdf_resp = match parsed_pdf_resp {
                Ok(parsed_pdf_resp) => parsed_pdf_resp,
                Err(e) => {
                    error!(%e,"task join error");
                    return Err("Internal error".to_owned());
                }
            };

            let ticket_data = match parsed_pdf_resp {
                Ok(data) => data,
                Err(e) => {
                    let user_error_message = match e {
                        pdf_parser::ParsePdfError::PdfExtractError(err) => {
                            trace!(%err, "parsing departure data in telegram receiver");
                            EXTRACT_TICKET_ERROR_MESSAGE
                        }
                        err => {
                            error!(%err, "parsing departure data in telegram receiver");
                            LAYOUT_CHANGED_MESSAGE
                        }
                    };

                    return Err(user_error_message.to_owned());
                }
            };

            if let Err(e) = db.insert_ticket_data(user, ticket_data.clone()) {
                error!(%e,"inserting to db");
                return Err("Database Error.".to_owned());
            } else {
                trace!(%user,?ticket_data, "inserted to db");
            }
            let message = format!(
            "Your ticket to train №{train_num}, departing at {depart_at}, is added to monitoring!",
            train_num = ticket_data.train_number,
            depart_at = ticket_data.departure_datetime
        );

            Ok(message)
        }
    })
    .await;
}

pub fn build_train_notification_message(
    user_ticket: TicketData,
    delayed_train: Option<&DelayedTrain>,
) -> String {
    if let Some(delayed_train) = delayed_train {
        let delay_minutes = delayed_train.delay.hr * 60 + delayed_train.delay.min;

        let train_probable_departure_time = user_ticket
            .departure_datetime
            .checked_add_signed(Duration::minutes(delay_minutes as i64))
            .expect("must not overflow")
            .time();
        format!("Your train №{train_number} {train_direction} is delayed by {delay_minutes} minutes.\nProbable arrival time is {arrival_hr:0>2}:{arrival_min:0>2} Kyiv time.",train_number=user_ticket.train_number,train_direction=delayed_train.direction,arrival_hr=train_probable_departure_time.hour(),arrival_min=train_probable_departure_time.minute())
    } else {
        let train_probable_departure_time = user_ticket.departure_datetime.time();

        format!("No delays found for your train №{train_number}!\nArrival time is {arrival_hr:0>2}:{arrival_min:0>2} Kyiv time.",train_number=user_ticket.train_number,arrival_hr=train_probable_departure_time.hour(),arrival_min=train_probable_departure_time.minute())
    }
}
