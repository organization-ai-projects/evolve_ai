use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let msg_input = use_state(|| String::new());
    let log = use_state(|| Vec::<String>::new());
    // Writer WebSocket dans un Rc<RefCell<Option<...>>>
    let ws_writer = use_state(|| Rc::new(RefCell::new(None)));

    let on_connect = {
        let ws_writer = ws_writer.clone();
        let log = log.clone();
        Callback::from(move |_| {
            let ws_writer = ws_writer.clone();
            let log = log.clone();
            spawn_local(async move {
                let (write, mut read) = match WebSocket::open("ws://127.0.0.1:9001") {
                    Ok(ws) => ws.split(),
                    Err(_) => {
                        log.set({
                            let mut l = (*log).clone();
                            l.push("Erreur de connexion WebSocket".to_owned());
                            l
                        });
                        return;
                    }
                };

                log.set({
                    let mut l = (*log).clone();
                    l.push("[Connecté au serveur WS]".to_owned());
                    l
                });

                // Receiver task
                let log_reader = log.clone();
                spawn_local(async move {
                    while let Some(msg) = read.next().await {
                        if let Ok(Message::Text(txt)) = msg {
                            log_reader.set({
                                let mut l = (*log_reader).clone();
                                l.push(format!("Serveur: {txt}"));
                                l
                            });
                        }
                    }
                });

                // Enregistre le writer dans le state global pour envoyer après
                *ws_writer.borrow_mut() = Some(write);
            });
        })
    };

    let on_send = {
        let ws_writer = ws_writer.clone();
        let msg_input = msg_input.clone();
        let log = log.clone();
        Callback::from(move |_| {
            let msg = (*msg_input).clone();
            let log2 = log.clone();
            let ws_writer2 = ws_writer.clone();
            spawn_local(async move {
                let mut writer_ref = ws_writer2.borrow_mut();
                if let Some(ws_writer) = writer_ref.as_mut() {
                    if ws_writer.send(Message::Text(msg.clone())).await.is_ok() {
                        log2.set({
                            let mut l = (*log2).clone();
                            l.push(format!("Moi: {msg}"));
                            l
                        });
                    }
                }
            });
            msg_input.set(String::new());
        })
    };

    let on_input = {
        let msg_input = msg_input.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(val) = e
                .target_dyn_into::<web_sys::HtmlInputElement>()
                .map(|v| v.value())
            {
                msg_input.set(val);
            }
        })
    };

    html! {
        <div style="max-width:500px;margin:2em auto; background:#fff; color:#222;">
            <h1>{ "Yew + Rust WebSocket Demo" }</h1>
            <button onclick={on_connect}>{ "Se connecter WebSocket" }</button>
            <div style="margin:1em 0;">
                <input type="text" value={(*msg_input).clone()} oninput={on_input} />
                <button onclick={on_send}>{ "Envoyer" }</button>
            </div>
            <div style="background:#222;color:#ddd;padding:1em;border-radius:1em;min-height:8em;">
                { for (*log).iter().rev().map(|line| html!{<div>{ line }</div>}) }
            </div>
        </div>
    }
}

// Main compatible CSR
#[cfg(feature = "csr")]
fn main() {
    yew::Renderer::<App>::with_root(
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("root")
            .unwrap(),
    )
    .render();
}

#[cfg(not(feature = "csr"))]
fn main() {}
