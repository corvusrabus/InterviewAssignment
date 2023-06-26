use crate::defines::grpc_scheme::orderbook_aggregator_server::OrderbookAggregator;
use crate::defines::grpc_scheme::{Empty, Summary};
use crate::defines::Exchange;
use crate::feed::OrderbookFeedFactory;
use crate::marketdata::BookAggregatorCallback;
use async_broadcast::{InactiveReceiver, Receiver};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct BookSummaryService {
    receiver: InactiveReceiver<Result<Summary, tonic::Status>>,
}

impl BookSummaryService {
    // Number of updates a client can lag behind before server will start dropping
    // old messages
    const BUFFER_SIZE: usize = 50;
    pub fn new(symbol: &str) -> Self {
        let (mut sender, rx) = async_broadcast::broadcast(Self::BUFFER_SIZE);
        sender.set_overflow(true);
        let receiver = rx.deactivate();
        let callback = Arc::new(BookAggregatorCallback::new(sender.clone()));
        let mut binance_feed =
            OrderbookFeedFactory::create_feed(Exchange::Binance, callback.clone());
        let mut bitstamp_feed = OrderbookFeedFactory::create_feed(Exchange::Bitstamp, callback);
        let symbol_clone1 = symbol.to_string();
        let symbol_clone2 = symbol.to_string();
        tokio::spawn(async move { bitstamp_feed.start(&symbol_clone1).await });
        tokio::spawn(async move { binance_feed.start(&symbol_clone2).await });

        Self { receiver }
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for BookSummaryService {
    type BookSummaryStream = Receiver<Result<Summary, tonic::Status>>;
    async fn book_summary(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::BookSummaryStream>, tonic::Status> {
        Ok(tonic::Response::new(self.receiver.activate_cloned()))
    }
}
