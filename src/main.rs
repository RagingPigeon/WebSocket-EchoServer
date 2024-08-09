use tokio_tungstenite;
use tracing::{event, span, Level};
use tracing_subscriber;
use warp::Filter;
use futures_util::{FutureExt, StreamExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Configure WebSocket

    let routes = warp::path("echo")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|websocket| {
                
                
                let (tx, rx) = websocket.split();

                

                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        event!(Level::DEBUG, "websocket error: {:?}", e);
                    }
                })
            })
        });

    // Listen for messages
    // Echo received message
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}