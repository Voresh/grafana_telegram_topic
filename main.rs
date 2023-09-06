use warp::{Filter, Reply, Rejection};
use serde::Deserialize;
use std::collections::HashMap;
use warp::hyper::{Client, Uri, Body};
use hyper_tls::HttpsConnector;
use config::Config;

#[derive(Deserialize, Debug)]
pub struct Notification {
    pub status: String,
    pub alerts: Vec<Alert>,
    #[serde(rename = "externalURL")]
    pub external_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Alert {
    pub status: String,
    pub labels: HashMap<String, String>,
}

pub async fn handle_notification(notification: Notification, settings: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let mut message = String::new();
    for alert in &notification.alerts {
        message.push_str(&format!("{}\n", alert.status));
        for (key, value) in &alert.labels {
            message.push_str(&format!("{}: {}\n", key, value));
        }
    }
    message.push_str(&notification.external_url);
    let token = &settings["token"];
    let chat_id = &settings["chat_id"];
    let message_thread_id = &settings["message_thread_id"];
    let escaped_message = url_escape::encode_fragment(&message);
    let uri_string = format!("https://api.telegram.org/bot{token}/sendMessage?chat_id={chat_id}&message_thread_id={message_thread_id}&text={escaped_message}&parse_mode=html");
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);
    let uri: Result<Uri, _> = uri_string.parse();
    if uri.is_err() {
        return Err(warp::reject::reject());
    }
    let uri_unwrapped = uri.unwrap();
    let res = client.get(uri_unwrapped).await;
    if res.is_err() {
        return Err(warp::reject::reject());
    }
    return Ok(warp::reply::html("ok!"));
}

fn with_settings(settings: HashMap<String, String>) -> impl Filter<Extract = (HashMap<String, String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || settings.clone())
}

#[tokio::main]
async fn main() {
    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();
    let settings = config
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();
    // curl -X POST http://172.25.112.1:3030/notify -H "Content-Type: application/json" -d @test.json 
    let alert = warp::path("notify")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_settings(settings))
        .and_then(handle_notification);
    warp::serve(alert)
        .run(([0, 0, 0, 0], 3030))
        .await;
}