use tonic::Request;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

use orderbook::{orderbook_aggregator_client::OrderbookAggregatorClient, Empty};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:1079").await?;

    let mut stream = client
        .book_summary(Request::new(Empty {}))
        .await?
        .into_inner();

    while let Some(levels) = stream.message().await? {
        println!("BookSummary = {:?}", levels);
    }
    Ok(())
}
