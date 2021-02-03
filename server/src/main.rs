extern crate futures;
extern crate parking_lot;
extern crate sqlx;
extern crate tokio;
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
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|websocket| {
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

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

async fn sensor_connected(ws: WebSocket, users: Users, database: Database) {
}
