use std::time::Duration;
use log::LevelFilter::Info;
use tokio::sync::mpsc::channel;
use tokio::time::sleep;
use crate::defines::Exchange;
use crate::feed::OrderbookFeedFactory;
use crate::marketdata::BookAggregator;

mod feed;
mod defines;
mod marketdata;
pub(crate) mod helper;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // env_logger::init();
    env_logger::builder().filter_level(Info).init();
    let (tx, mut rx) = channel(20);
    let mut binance_feed = OrderbookFeedFactory::create_feed(Exchange::Binance, tx.clone());
    let mut bitstamp_feed = OrderbookFeedFactory::create_feed(Exchange::Bitstamp, tx);
    let mut aggregator = BookAggregator::new();
    tokio::spawn(async move {bitstamp_feed.start("btcusdt").await});
    tokio::spawn(async move {binance_feed.start("btcusdt").await});
    println!("Started");
    while let Some((exchange, book)) = rx.recv().await {
        aggregator.add_new_book(exchange, book);
        println!("{:?}", aggregator.make_grpc_message())
    }

}

