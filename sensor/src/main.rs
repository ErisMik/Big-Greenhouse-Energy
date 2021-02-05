extern crate chrono;
extern crate futures;
extern crate futures_util;
extern crate protobuf;
extern crate tokio;
extern crate tokio_tungstenite;

use futures_util::{future, pin_mut, StreamExt, SinkExt};
use protobuf::Message;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite};

mod protos;

#[tokio::main]
async fn main() {
    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    while let Some(message) = ws_stream.next().await {
        let message_data = message.unwrap().into_data();

        if let Ok(request) = protos::sensor::SensorRequest::parse_from_bytes(&message_data) {
            println!("Recived request {:?}", request);

            let temp = 45.2;
            let timestamp = chrono::Utc::now().timestamp();

            let mut response_header = protos::sensor::SensorResponseHeader::new();
            response_header.set_requestID(request.requestID);
            response_header.set_dataTimestamp(timestamp);

            let mut response = protos::sensor::ThermometerResponse::new();
            response.set_header(response_header);
            response.set_temperatureCelcius(temp);

            if let Ok(response_bytes) = response.write_to_bytes() {
                let response_message = tungstenite::Message::Binary(response_bytes);
                ws_stream.send(response_message).await;
            }
        }
    }
}
