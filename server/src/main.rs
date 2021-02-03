use futures::{FutureExt, StreamExt};
use warp::Filter;

#[tokio::main]
async fn main() {
    let index = warp::get()
        .and(warp::fs::dir("../viewer/build"));

    let websocket = warp::path("ws").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                if let Err(e) = result {
                    eprintln!("websocket error: {:?}", e);
                }
            })
        })
    });

    let routes = index.or(websocket);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
