use crate::book_service::orderbook_aggregator_client::OrderbookAggregatorClient;
use crate::book_service::Empty;
use futures_util::StreamExt;
pub mod book_service {
    tonic::include_proto!("orderbook");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let mut client = OrderbookAggregatorClient::connect("http://127.0.0.1:8080")
        .await
        .unwrap();
    let summary_stream = client.book_summary(Empty {}).await?;
    println!("{summary_stream:?}");
    let mut stream = summary_stream.into_inner();
    while let Some(stuff) = stream.next().await {
        println!("{stuff:?}");
    }
    Ok(())
}
