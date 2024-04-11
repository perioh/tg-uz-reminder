mod consts;
mod delayed_trains;
mod errors;
pub use delayed_trains::DelayedTrain;
use delayed_trains::DelayedTrains;
pub use errors::UzParseError;
use reqwest::{header::HeaderMap, Client, ClientBuilder, Url};
use tracing::trace;

pub struct UzParserClient {
    web_client: Client,
    url: Url,
}

impl UzParserClient {
    pub fn new() -> Self {
        let url: Url = consts::UZ_DELAYS_URL_STR.parse().expect("Wrong uz url");
        let web_client = ClientBuilder::new()
            .default_headers(uz_default_headers())
            .build()
            .expect("Error building uz web client");

        Self { url, web_client }
    }

    pub async fn delayed_trains(&self) -> Result<DelayedTrains, UzParseError> {
        let html_body = self.delays_page_content().await?;
        let delayed_trains: DelayedTrains = html_body.parse()?;
        trace!(?delayed_trains);
        Ok(delayed_trains)
    }

    async fn delays_page_content(&self) -> Result<String, UzParseError> {
        let resp_body_html = self
            .web_client
            .get(self.url.clone())
            .send()
            .await?
            .text()
            .await?;

        Ok(resp_body_html)
    }

}

fn uz_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.append("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().expect("Eror parsing Accept header"));
    headers.append("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36".parse().expect("Eror parsing User-Agent header"));
    headers
}

#[cfg(test)]
mod tests {
    use crate::delayed_trains::DelayedTrains;

    #[test]
    fn test_page_parser() {
        let input = r###"<!DOCTYPE html>
        <html lang="uk">
        <head>
            <meta charset="utf-8">
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
            <meta http-equiv="X-UA-Compatible" content="IE=edge">
            <meta http-equiv="cleartype" content="on">
            <meta name="MobileOptimized" content="320">
            <meta name="HandheldFriendly" content="True">
            <meta name="apple-mobile-web-app-capable" content="yes">
            <meta name="viewport" content="width=device-width, initial-scale=1">
        
            <link rel="icon" type="image/png" href="/frontend/images/favicon.png" sizes="16x16">
        
            <meta property="og:image" content="/frontend/images/og_image.jpg">
            <meta property="og:type" content="website" />
            <meta property="og:title" content="Поїзди що затримуються" />
            <meta property="og:description" content="Поїзди що затримуються" />
            <meta property="og:url" content="http://uz-vezemo.uz.gov.ua/delayform" />
            <meta property="og:site_name" content="uz-vezemo.com" />
        
            <meta name="theme-color" content="#262a82">
        
            <title>Поїзди що затримуються</title>
            <meta name="description" content="Поїзди що затримуються">
        
            
            <link href="/frontend/css/app.css" rel="stylesheet"/>
            <link href="/frontend/css/jquery-ui.css" rel="stylesheet"/>
        
            <meta name="csrf-token" content="xummgynXxfvMA7cct0hOXYmgGEbE1IWBLxXxQOx4">
                
            <script async src="https://www.googletagmanager.com/gtag/js?id=G-S5W7SHH1G8"></script>
            <script>
                window.dataLayer = window.dataLayer || [];
        
                function gtag() {
                    dataLayer.push(arguments);
                }
        
                gtag("js", new Date());
        
                gtag("config", "G-S5W7SHH1G8");
            </script>
        </head>
        
        <body class="delayform">
        
        
        
        
        
        
        
        
        
        <div class="wrapper delayform-page">
                <main>
                <section class="delayform-wrapper">
                    <a href="https://uz-vezemo.uz.gov.ua" class="arrow-heroe">
                        <img src="/frontend/images/arrow-heroe-back.png" alt="arrow"/> </a>
                    <div class="container">
                        <div class="delayform__in">
                            <div class="delayform-title">Затримуються наступні поїзди</div>
                            
        
        
                                <ul class="delayform-list ">
        
                                                                    <li>№705/706 Пшемисль Головний-Київ-Пас. (+0:30)</li>
                                                                    <li>№749/750 Київ-Пас.-Відень Головний (+0:11)</li>
                                                                    <li>№721/722 Київ-Пас.-Харків-Пас. (+0:09)</li>
                                                            </ul>
                                                <div class="search-delays--js"></div>
                        </div>
                    </div>
                </section>
            </main>
        </div>
        
        <script src="/frontend/js/jq.js"></script>
        <script src="/frontend/js/jquery-ui.js"></script>
        <script src="/frontend/js/datepicker-uk.js"></script>
        <script src="/frontend/js/main.js"></script>
        <script src="/frontend/js/search-train-form.js"></script>
        
        <script src="/frontend/js/delays.js"></script>
        </body>
        </html>"###;

        let trains: DelayedTrains = input.parse().expect("Error parsing input");

        assert_eq!(
            format!("{trains:?}"),
            r#"DelayedTrains([DelayedTrain { direction: TrainDirection("Пшемисль Головний-Київ-Пас."), numbers: TrainNumbers([705, 706]), delay: TrainDelayTime { hr: 0, min: 30 } }, DelayedTrain { direction: TrainDirection("Київ-Пас.-Відень Головний"), numbers: TrainNumbers([749, 750]), delay: TrainDelayTime { hr: 0, min: 11 } }, DelayedTrain { direction: TrainDirection("Київ-Пас.-Харків-Пас."), numbers: TrainNumbers([721, 722]), delay: TrainDelayTime { hr: 0, min: 9 } }])"#
        );
    }
}
