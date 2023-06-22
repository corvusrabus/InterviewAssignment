mod exchanges;
mod ws_api_feed;
mod orderbook_feed;

use tokio::sync::mpsc::Sender;
use tokio::task::JoinError;
use crate::defines::Exchange;
use crate::feed::exchanges::binance::BinanceOrderbookWsApi;
use crate::feed::exchanges::bitstamp::BitstampOrderbookWsApi;
use crate::feed::orderbook_feed::ExchangeOrderbookFeed;
use crate::marketdata::Orderbook;

#[async_trait::async_trait]
pub trait OrderbookFeed : Sync + Send {
    async fn start(&mut self, symbol: &str);
    async fn join(&mut self) -> Result<(), JoinError>;
}

pub struct OrderbookFeedFactory {}

impl OrderbookFeedFactory {
    pub(crate) fn create_feed(exchange: Exchange,sender : Sender<(Exchange,Orderbook)>) -> Box<dyn OrderbookFeed> {
        match exchange {
            Exchange::Binance => {
                Box::new(ExchangeOrderbookFeed::<BinanceOrderbookWsApi>::new(sender))
            }
            Exchange::Bitstamp => {
                Box::new(ExchangeOrderbookFeed::<BitstampOrderbookWsApi>::new(sender))

            }
        }
    }
}