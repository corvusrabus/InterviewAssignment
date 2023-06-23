use crate::defines::book_callback::BookCallback;
use crate::defines::Exchange;
use crate::feed::ws_api_feed::{OrderbookWebsocket, OrderbookWsApi};
use crate::feed::OrderbookFeed;
use crate::helper::{is_ascending_by_key, is_descending_by_key};
use crate::marketdata::Orderbook;
use log::info;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::{JoinError, JoinHandle};
use tokio::time::sleep;

pub(in crate::feed) struct ExchangeOrderbookFeed<ExchangeApi: OrderbookWsApi, T: BookCallback> {
    handle: Option<JoinHandle<()>>,
    phantom: PhantomData<ExchangeApi>,
    callback: Option<T>,
}

impl<ExchangeApi: OrderbookWsApi, T: BookCallback> ExchangeOrderbookFeed<ExchangeApi, T> {
    pub fn new(callback: T) -> Self {
        Self {
            handle: None,
            phantom: Default::default(),
            callback: Some(callback),
        }
    }
}

#[async_trait::async_trait]
impl<ExchangeApi: OrderbookWsApi, T: BookCallback> OrderbookFeed
    for ExchangeOrderbookFeed<ExchangeApi, T>
{
    async fn start(&mut self, symbol: &str) {
        let symbol = String::from(symbol);
        // Check if already running
        if self.handle.is_some() || self.callback.is_none() {
            return;
        }
        let sender = self.callback.take().unwrap();
        let handle = tokio::spawn(async move {
            let exchange = ExchangeApi::exchange();
            loop {
                let mut ws =
                    match OrderbookWebsocket::<ExchangeApi>::connect_and_subscribe(&symbol).await {
                        Ok(ws) => ws,
                        Err(e) => {
                            info!(target : "OrderbookFeed", "Unexpected error {e:?}");
                            // wait 1s to not get rate limited
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };
                loop {
                    match ws.next_book().await {
                        Ok(book) => {
                            sender.accept_book(book, exchange).await;
                        }
                        Err(e) => {
                            info!(target : "OrderbookFeed", "Unexpected error {e:?}");
                            break;
                        }
                    }
                }
            }
        });
        self.handle = Some(handle);
    }

    async fn join(&mut self) -> Result<(), JoinError> {
        if let Some(handle) = self.handle.take() {
            handle.await
        } else {
            Ok(())
        }
    }
}
