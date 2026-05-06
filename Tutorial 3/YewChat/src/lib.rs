#![recursion_limit = "512"]

use futures::{SinkExt, StreamExt};
use reqwasm::websocket::{futures::WebSocket, Message as WsMessage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct ChatMessage {
    from: String,
    message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerMessage {
    message_type: String,
    data: Option<String>,
    data_array: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientMessage {
    message_type: String,
    data: String,
    data_array: Vec<String>,
}

enum Msg {
    SetNickname(String),
    SetDraft(String),
    Connect,
    SendMessage,
    WsMessage(String),
    WsReady(futures::stream::SplitSink<WebSocket, WsMessage>),
    WsClosed,
}

struct App {
    nickname: String,
    draft: String,
    users: Vec<String>,
    messages: Vec<ChatMessage>,
    ws: Option<futures::stream::SplitSink<WebSocket, WsMessage>>,
    connected: bool,
    error: Option<String>,
}

impl App {
    fn client_message(message_type: &str, data: String) -> WsMessage {
        let payload = ClientMessage {
            message_type: message_type.to_owned(),
            data,
            data_array: Vec::new(),
        };

        WsMessage::Text(serde_json::to_string(&payload).unwrap())
    }

    fn connection_copy(&self) -> &'static str {
        if self.connected {
            "Live connection"
        } else {
            "Waiting for server"
        }
    }

    fn view_signal_icon() -> Html {
        html! {
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 17h3v3H4z" />
                <path d="M10.5 12h3v8h-3z" />
                <path d="M17 5h3v15h-3z" />
            </svg>
        }
    }

    fn view_send_icon() -> Html {
        html! {
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M3 11.2 20.2 3.4c.7-.3 1.4.4 1.1 1.1l-7.8 17.2c-.3.8-1.5.7-1.7-.1l-1.7-7.1-7.1-1.7c-.8-.2-.9-1.3-.1-1.6z" />
                <path d="m10.3 14.2 4.3-4.3" />
            </svg>
        }
    }

    fn view_chat_icon() -> Html {
        html! {
            <svg class="hero-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M5 6.5A3.5 3.5 0 0 1 8.5 3h7A3.5 3.5 0 0 1 19 6.5v5a3.5 3.5 0 0 1-3.5 3.5H12l-4.2 3.3A1.1 1.1 0 0 1 6 17.4V15A3.5 3.5 0 0 1 5 12.5z" />
                <path d="M8.5 8h7" />
                <path d="M8.5 11h4.5" />
            </svg>
        }
    }

    fn view_lobby(&self, ctx: &Context<Self>, on_nickname: Callback<InputEvent>) -> Html {
        html! {
            <main class="app-shell lobby-shell">
                <section class="lobby-panel" aria-label="Join chat">
                    <div class="brand-row compact">
                        <span class="brand-mark">{ Self::view_chat_icon() }</span>
                        <span>{"YewChat"}</span>
                    </div>
                    <div class="status-card">
                        <span class={classes!("status-dot", if self.connected { "is-live" } else { "is-waiting" })}></span>
                        <span>{ self.connection_copy() }</span>
                    </div>
                    <h2>{"Enter the room"}</h2>
                    <p>{"Choose a nickname before connecting to ws://127.0.0.1:8080."}</p>
                    <input
                        class="text-input"
                        oninput={on_nickname}
                        placeholder="Nickname"
                        value={self.nickname.clone()}
                    />
                    <button class="primary-button" onclick={ctx.link().callback(|_| Msg::Connect)}>
                        { Self::view_signal_icon() }
                        <span>{"Connect"}</span>
                    </button>
                    if let Some(error) = &self.error {
                        <p class="error-text">{ error }</p>
                    }
                </section>
            </main>
        }
    }

    fn view_chat(
        &self,
        ctx: &Context<Self>,
        on_draft: Callback<InputEvent>,
        on_keypress: Callback<KeyboardEvent>,
    ) -> Html {
        html! {
            <main class="app-shell chat-shell">
                <header class="chat-header">
                    <div>
                        <div class="brand-row compact">
                            <span class="brand-mark">{ Self::view_chat_icon() }</span>
                            <span>{"YewChat"}</span>
                        </div>
                        <p>{"Signed in as "}<strong>{ &self.nickname }</strong></p>
                    </div>
                    <div class="status-card">
                        <span class="status-dot is-live"></span>
                        <span>{ self.connection_copy() }</span>
                    </div>
                </header>

                if let Some(error) = &self.error {
                    <p class="error-banner">{ error }</p>
                }

                <section class="workspace">
                    <section class="message-panel" aria-label="Messages">
                        <div class="message-list">
                            if self.messages.is_empty() {
                                <div class="empty-state">
                                    <span class="empty-icon">{ Self::view_send_icon() }</span>
                                    <h2>{"The room is quiet."}</h2>
                                    <p>{"Send the first message, then open another browser tab to see the asynchronous broadcast."}</p>
                                </div>
                            } else {
                                { for self.messages.iter().map(|message| {
                                    let own_message = message.from == self.nickname;
                                    html! {
                                        <article class={classes!("message-bubble", if own_message { "own-message" } else { "peer-message" })}>
                                            <div class="message-author">{ &message.from }</div>
                                            <p>{ &message.message }</p>
                                        </article>
                                    }
                                }) }
                            }
                        </div>
                        <div class="composer">
                            <input
                                class="text-input"
                                oninput={on_draft}
                                onkeypress={on_keypress}
                                placeholder="Write a message"
                                value={self.draft.clone()}
                            />
                            <button class="icon-button" onclick={ctx.link().callback(|_| Msg::SendMessage)} title="Send message">
                                { Self::view_send_icon() }
                            </button>
                        </div>
                    </section>

                    <aside class="side-panel" aria-label="Online users">
                        <h2>{"Online now"}</h2>
                        <p>{"Every nickname below arrived through the server broadcast."}</p>
                        <ul class="user-list">
                            { for self.users.iter().map(|user| html! {
                                <li>
                                    <span class="avatar">{ user.chars().next().unwrap_or('U').to_ascii_uppercase() }</span>
                                    <span>{ user }</span>
                                </li>
                            }) }
                        </ul>
                    </aside>
                </section>
            </main>
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            nickname: String::new(),
            draft: String::new(),
            users: Vec::new(),
            messages: Vec::new(),
            ws: None,
            connected: false,
            error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetNickname(nickname) => {
                self.nickname = nickname;
                true
            }
            Msg::SetDraft(draft) => {
                self.draft = draft;
                true
            }
            Msg::Connect => {
                let nickname = self.nickname.trim().to_owned();
                if nickname.is_empty() {
                    self.error = Some("Please enter a nickname first.".to_owned());
                    return true;
                }

                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match WebSocket::open("ws://127.0.0.1:8080") {
                        Ok(ws) => {
                            let (mut write, mut read) = ws.split();
                            if write
                                .send(App::client_message("register", nickname))
                                .await
                                .is_ok()
                            {
                                link.send_message(Msg::WsReady(write));
                            }

                            while let Some(message) = read.next().await {
                                match message {
                                    Ok(WsMessage::Text(text)) => {
                                        link.send_message(Msg::WsMessage(text));
                                    }
                                    Ok(WsMessage::Bytes(_)) => {}
                                    Err(_) => {
                                        link.send_message(Msg::WsClosed);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(_) => link.send_message(Msg::WsClosed),
                    }
                });

                self.error = None;
                false
            }
            Msg::SendMessage => {
                let draft = self.draft.trim().to_owned();
                if draft.is_empty() {
                    return false;
                }

                if let Some(mut ws) = self.ws.take() {
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if ws.send(App::client_message("message", draft)).await.is_ok() {
                            link.send_message(Msg::WsReady(ws));
                        } else {
                            link.send_message(Msg::WsClosed);
                        }
                    });
                    self.draft.clear();
                    true
                } else {
                    self.error = Some("Websocket is not connected.".to_owned());
                    true
                }
            }
            Msg::WsMessage(text) => {
                if let Ok(message) = serde_json::from_str::<ServerMessage>(&text) {
                    match message.message_type.as_str() {
                        "users" => {
                            self.users = message.data_array.unwrap_or_default();
                        }
                        "message" => {
                            if let Some(data) = message.data {
                                if let Ok(chat_message) = serde_json::from_str::<ChatMessage>(&data)
                                {
                                    self.messages.push(chat_message);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                true
            }
            Msg::WsReady(ws) => {
                self.ws = Some(ws);
                self.connected = true;
                self.error = None;
                true
            }
            Msg::WsClosed => {
                self.ws = None;
                self.connected = false;
                self.error = Some("Unable to connect to ws://127.0.0.1:8080.".to_owned());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_nickname = ctx.link().callback(|event: InputEvent| {
            let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
            Msg::SetNickname(input.value())
        });
        let on_draft = ctx.link().callback(|event: InputEvent| {
            let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
            Msg::SetDraft(input.value())
        });
        let on_keypress = ctx.link().batch_callback(|event: KeyboardEvent| {
            if event.key() == "Enter" {
                Some(Msg::SendMessage)
            } else {
                None
            }
        });

        if self.connected {
            self.view_chat(ctx, on_draft, on_keypress)
        } else {
            self.view_lobby(ctx, on_nickname)
        }
    }
}

#[wasm_bindgen]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
