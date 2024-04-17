mod consts;
mod errors;

use consts::*;
pub use errors::TelegramErrors;
use futures::{Future, StreamExt};
use reqwest::Url;
use reqwest::{header::HeaderMap, Client, ClientBuilder};
use teloxide::requests::Requester;
pub use teloxide::types::ChatId;
use teloxide::RequestError;
use teloxide::{net::Download, types::Document};
use teloxide::{types::Message, Bot};
use tracing::trace;

#[derive(Clone)]
pub struct TelegramClient(Bot, Client);
impl TelegramClient {
    pub fn new(token: impl Into<String>) -> Self {
        let bot = teloxide::Bot::new(token);
        let web_client = ClientBuilder::new()
            .default_headers(uz_default_headers())
            .build()
            .expect("Error building uz file downloader web client");
        Self(bot, web_client)
    }

    pub async fn send_to_user(
        &self,
        user_id: ChatId,
        message: impl Into<String>,
    ) -> Result<(), TelegramErrors> {
        let _ = self.0.send_message(user_id, message).await?;
        Ok(())
    }

    pub async fn receive_messages<F, O>(self, parse: F)
    where
        F: Fn(ChatId, Vec<u8>) -> O + Clone + Send + Sync + 'static,
        O: Future<Output = Result<String, String>> + Send,
    {
        teloxide::repl(self.0, move |bot: Bot, msg: Message| {
            let client = self.1.clone();
            let parse = parse.clone();
            async move {
                if msg.text() == Some("/start") {
                    bot.send_message(msg.chat.id, WELCOME_MESSAGE).await?;
                    trace!(%msg.chat.id, "welcome");
                    return Ok(());
                }

                let file_content = if let Some(document) = msg.document() {
                    download_telegram_document(&bot, document).await?
                } else if msg
                    .text()
                    .map(|text| text.starts_with("https://app.uz.gov.ua/ticket-"))
                    .unwrap_or(false)
                {
                    let Ok(url) = msg.text().unwrap_or_default().parse::<Url>() else {
                        bot.send_message(msg.chat.id, URL_PARSE_ERROR).await?;
                        return Ok(());
                    };

                    download_uz_document(&client, url).await?
                } else {
                    bot.send_message(msg.chat.id, WRONG_MESSAGE_RECEIVED)
                        .await?;
                    trace!(%msg.chat.id, "message without document");
                    return Ok(());
                };

                let message = match parse(msg.chat.id, file_content).await {
                    Ok(data) => data,
                    Err(user_error_message) => {
                        bot.send_message(msg.chat.id, user_error_message).await?;
                        return Ok(());
                    }
                };
                bot.send_message(msg.chat.id, message).await?;
                trace!(%msg.chat.id,"user notified about new train added");
                Ok(())
            }
        })
        .await;
    }
}

async fn download_telegram_document(
    bot: &Bot,
    document: &Document,
) -> Result<Vec<u8>, RequestError> {
    let file = bot.get_file(&document.file.id).await?;
    let mut stream = bot.download_file_stream(&file.path);
    let mut file_content = vec![];
    while let Some(Ok(chunk)) = stream.next().await {
        file_content.extend_from_slice(&chunk);
    }
    Ok(file_content)
}
async fn download_uz_document(client: &Client, path: Url) -> Result<Vec<u8>, reqwest::Error> {
    let file = client.get(path).send().await?;

    let bytes = file.bytes().await?;

    Ok(bytes.to_vec())
}

fn uz_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.append("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().expect("Eror parsing Accept header"));
    headers.append("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36".parse().expect("Eror parsing User-Agent header"));
    headers
}
