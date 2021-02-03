extern crate futures;
extern crate parking_lot;
extern crate sqlx;
extern crate tokio;
extern crate tokio_stream;
extern crate warp;

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::{FutureExt, StreamExt};
use parking_lot::Mutex;
use sqlx::{Connection, SqliteConnection};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_SENSOR_ID: AtomicUsize = AtomicUsize::new(1);

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;
type Sensors = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;
type Database = Arc<Mutex<SqliteConnection>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = SqliteConnection::connect("sqlite::memory:").await?; // Switch to pool with SQLx 0.5
    let database = Arc::new(Mutex::new(conn));
    let database = warp::any().map(move || database.clone());

    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    let viewer_websocket = warp::path("viewerws")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| user_connected(socket, users)));

    let sensors = Sensors::default();
    let sensors = warp::any().map(move || sensors.clone());

    let sensor_websocket = warp::path("sensorws")
        .and(warp::ws())
        .and(sensors)
        .and(database)
        .map(|ws: warp::ws::Ws, sensors, database| {
            ws.on_upgrade(move |socket| sensor_connected(socket, sensors, database))
        });

    let index = warp::get().and(warp::fs::dir("../viewer/build"));

    let routes = index.or(viewer_websocket).or(sensor_websocket);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}

async fn user_connected(ws: WebSocket, users: Users) {
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    println!("User {} connected", my_id);

    let (user_ws_tx, mut user_ws_rx) = ws.split();

    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    users.write().await.insert(my_id, tx);

    let users_disconnect = users.clone();

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    user_disconnected(my_id, &users_disconnect).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        println!("Message \"{:?}\" not string", msg);
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);
    println!("Sending message {}", new_msg);

    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Ok(Message::text(new_msg.clone()))) {}
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    println!("User {} disconnected", my_id);
    users.write().await.remove(&my_id);
}

async fn sensor_connected(ws: WebSocket, users: Users, database: Database) {}
