use tonic::{transport::Server, Request, Response, Status};

use tokio_stream::wrappers::ReceiverStream;
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Empty, Summary};

pub mod orderbook {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyOrderbookAggregator {
    // register to book for updates
}

#[tonic::async_trait]
impl OrderbookAggregator for MyOrderbookAggregator {

    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // read commandline configuration currency pair

    // create queue (mpsc)

    // initialize book object
        // expose method for register subscribers
        // dequeue orders from queue
        // merge into BinaryHeap

    // binance driver
        // connects to exchange
        // receives orders, normalize and push into queue

    // bitstamp driver
        // connects to exchange
        // receives orders, normalize and push into queue


    // aggregator
        // register for updates to book
        // publish summary on grpc
    let aggregator = MyOrderbookAggregator::default();

    // setup address for grpc server binding
    let addr = "[::1]:50051".parse()?;

    // mainLoop: run grpc server
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;

    Ok(())
}
