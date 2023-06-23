use crate::defines::error::{WebsocketError, WebsocketResult};
use crate::defines::json_parser::JSONError;
use crate::defines::Exchange;
use crate::marketdata::Orderbook;
use futures_util::{SinkExt, StreamExt};
use log::info;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

pub(in crate::feed) trait OrderbookWsApi: Send + Sync {
    fn subscription_message(symbol: &str) -> String;
    /// Returns true if `message` confirms orderbook subscription for `symbol`
    fn verify_confirmation(symbol: &str, message: &str) -> bool;
    fn handle_message(msg: &str) -> Result<Orderbook, JSONError>;
    ///
    fn connection_endpoint() -> Url;
    fn exchange() -> Exchange;
}

type WsStreamTT = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct OrderbookFeedUnsubscribed<ExchangeApi: OrderbookWsApi> {
    stream: Option<WsStreamTT>,
    api: PhantomData<ExchangeApi>,
}

pub(in crate::feed) struct OrderbookWebsocket<ExchangeApi: OrderbookWsApi> {
    stream: WsStreamTT,
    api: PhantomData<ExchangeApi>,
}

impl<ExchangeApi: OrderbookWsApi> OrderbookWebsocket<ExchangeApi> {
    pub async fn connect_and_subscribe(symbol: &str) -> WebsocketResult<Self> {
        let endpoint = ExchangeApi::connection_endpoint();
        info!(target : "OrderbookFeed", "Connecting to {:?} at {endpoint}", ExchangeApi::exchange() );
        let (mut stream, _) = connect_async(endpoint).await?;
        info!(target : "OrderbookFeed", "Connected to {:?}", ExchangeApi::exchange() );
        let subscription_message = ExchangeApi::subscription_message(symbol);
        stream.send(Message::text(subscription_message)).await?;
        // First message should be subscription confirmation, but we'll allow 10 messages
        let mut sub_confirmation_received = false;
        for _ in 0..10 {
            tokio::select! {
                _ = sleep(Duration::from_secs(15)) => {
                    return Err(WebsocketError::Timeout);
                },
                message = stream.next() => {
                    if message.is_none() {
                    return Err(WebsocketError::UnexpectedClosure);
                }
                let message = message.unwrap()?;
                match message {
                    Message::Text(txt_msg) => {
                        if ExchangeApi::verify_confirmation(symbol, &txt_msg) {
                          sub_confirmation_received = true; break;
                        }
                    }
                      // Tungstenite replies to pings by itself
                     Message::Ping(_) => {}
                    Message::Pong(_) => {}
                    Message::Close(_) => {
                        return Err(WebsocketError::UnexpectedClosure);
                    }
                    _ => {
                        info!(target : "OrderbookFeed", "Unexpected message {message:?}")
                    }
                }
                }
            }
        }
        if !sub_confirmation_received {
            Err(WebsocketError::NoConfirmationReceived)
        } else {
            info!(target : "OrderbookFeed", "Subscribed to {:?}", ExchangeApi::exchange() );
            Ok(Self {
                stream,
                api: Default::default(),
            })
        }
    }
    pub async fn next_book(&mut self) -> Result<Orderbook, WebsocketError> {
        loop {
            let message = self.stream.next().await;
            if message.is_none() {
                return Err(WebsocketError::UnexpectedClosure);
            }
            let inner_message = message.unwrap()?;
            match inner_message {
                Message::Text(txt_msg) => match ExchangeApi::handle_message(&txt_msg) {
                    Ok(book) => {
                        return Ok(book);
                    }
                    Err(e) => {
                        info!(target : "OrderbookFeed", "Unexpected JSON {txt_msg}, error: {e:?}")
                    }
                },
                // Tungstenite replies to pings by itself
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(_) => {
                    return Err(WebsocketError::UnexpectedClosure);
                }
                _ => {
                    info!(target : "OrderbookFeed", "Unexpected message {inner_message:?}")
                }
            }
        }
    }
}
