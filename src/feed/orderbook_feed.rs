use std::marker::PhantomData;
use std::time::Duration;
use log::info;
use tokio::sync::mpsc::Sender;
use tokio::task::{JoinError, JoinHandle};
use tokio::time::sleep;
use crate::defines::Exchange;
use crate::feed::OrderbookFeed;
use crate::feed::ws_api_feed::{OrderbookWebsocket, OrderbookWsApi};
use crate::helper::{is_ascending_by_key, is_descending_by_key};
use crate::marketdata::Orderbook;

pub(in crate::feed) struct ExchangeOrderbookFeed<ExchangeApi: OrderbookWsApi> {
    handle: Option<JoinHandle<()>>,
    phantom: PhantomData<ExchangeApi>,
    sender: Option<Sender<(Exchange, Orderbook)>>,
}

impl<ExchangeApi: OrderbookWsApi> ExchangeOrderbookFeed<ExchangeApi> {
    pub fn new(sender: Sender<(Exchange, Orderbook)>) -> Self {
        Self { handle: None, phantom: Default::default(), sender: Some(sender) }
    }
}

#[async_trait::async_trait]
impl<ExchangeApi: OrderbookWsApi> OrderbookFeed for ExchangeOrderbookFeed<ExchangeApi> {
    async fn start(&mut self, symbol: &str) {
        let symbol = String::from(symbol);
        // Check if already running
        if self.handle.is_some() || self.sender.is_none() {
            return;
        }
        let sender = self.sender.take().unwrap();
        let handle = tokio::spawn(async move {
            let exchange = ExchangeApi::exchange();
            loop {
                let mut ws = match OrderbookWebsocket::<ExchangeApi>::connect_and_subscribe(&symbol).await {
                    Ok(ws) => { ws }
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
                            sender.send((exchange, book)).await.unwrap();
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
