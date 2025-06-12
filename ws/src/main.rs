use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("WebSocket server listening on ws://127.0.0.1:9001");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let mut ws_stream = accept_async(stream).await.unwrap();
            println!("New client connected!");
            // On écoute les messages du client, et on répond "pong: ..." à chaque fois
            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(msg) => {
                        if msg.is_text() {
                            let txt = msg.to_text().unwrap();
                            println!("Received: {txt}");
                            ws_stream.send(format!("pong: {txt}").into()).await.unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }
        });
    }
}
