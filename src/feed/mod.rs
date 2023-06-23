mod exchanges;
mod orderbook_feed;
mod ws_api_feed;

use crate::defines::book_callback::BookCallback;
use crate::defines::Exchange;
use crate::feed::exchanges::binance::BinanceOrderbookWsApi;
use crate::feed::exchanges::bitstamp::BitstampOrderbookWsApi;
use crate::feed::orderbook_feed::ExchangeOrderbookFeed;
use crate::marketdata::Orderbook;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinError;

#[async_trait::async_trait]
pub trait OrderbookFeed: Sync + Send {
    async fn start(&mut self, symbol: &str);
    async fn join(&mut self) -> Result<(), JoinError>;
}

pub struct OrderbookFeedFactory {}

impl OrderbookFeedFactory {
    pub(crate) fn create_feed<T: BookCallback>(
        exchange: Exchange,
        callback: T,
    ) -> Box<dyn OrderbookFeed> {
        match exchange {
            Exchange::Binance => Box::new(ExchangeOrderbookFeed::<BinanceOrderbookWsApi, T>::new(
                callback,
            )),
            Exchange::Bitstamp => Box::new(
                ExchangeOrderbookFeed::<BitstampOrderbookWsApi, T>::new(callback),
            ),
        }
    }
}
