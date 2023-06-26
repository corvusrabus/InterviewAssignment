use crate::defines::grpc_scheme::orderbook_aggregator_server::OrderbookAggregatorServer;
use crate::grpc_server::BookSummaryService;
use log::LevelFilter::Info;
use std::error::Error;
use tonic::transport::Server;

mod defines;
mod feed;
mod grpc_server;
pub(crate) mod helper;
mod marketdata;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Symbol of market to subscribe to
    #[arg(short, long, default_value_t = String::from("btcusdt"))]
    symbol: String,

    /// Address of socket to use
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    address: String,
}

#[tokio::main]
async fn main() {
    let Args { symbol, address } = Args::parse();

    env_logger::builder().filter_level(Info).init();
    let address = address.parse().unwrap_or_else(|_| {
        panic!(
            "Provided address {} is not a a valid socket address",
            address
        )
    });
    let book_service = BookSummaryService::new(&symbol);

    if let Err(e) = Server::builder()
        .add_service(OrderbookAggregatorServer::new(book_service))
        .serve(address)
        .await
    {
        println!(
            "Failed to start gRPC server due to {} {}",
            e,
            e.source().map(|x| format!("{x}")).unwrap_or_default()
        )
    }
}
