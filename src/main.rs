use crate::defines::Exchange;
use crate::feed::OrderbookFeedFactory;
use crate::marketdata::{BookAggregator, BookAggregatorCallback};
use log::LevelFilter::Info;
use std::sync::Arc;

mod defines;
mod feed;
mod grpc_server;
pub(crate) mod helper;
mod marketdata;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    env_logger::builder().filter_level(Info).init();
    let (tx, mut rx) = tokio::sync::broadcast::channel(20);
    // let mut aggregator = BookAggregator::new();
    let callback = Arc::new(BookAggregatorCallback::new(tx));
    // let (tx, mut rx) = channel(20);
    let mut binance_feed = OrderbookFeedFactory::create_feed(Exchange::Binance, callback.clone());
    let mut bitstamp_feed = OrderbookFeedFactory::create_feed(Exchange::Bitstamp, callback);
    tokio::spawn(async move { bitstamp_feed.start("btcusdt").await });
    tokio::spawn(async move { binance_feed.start("btcusdt").await });
    println!("Started");
    // while let Some((exchange, book)) = rx.recv().await {
    //     aggregator.add_new_book( book,exchange);
    //     println!("{:?}", aggregator.make_grpc_message())
    // }
    loop {
        let msg = rx.recv().await;
        println!("{msg:?}");
        // aggregator.add_new_book(book, exchange);
        // println!("{:?}", aggregator.make_grpc_message())
    }
}
