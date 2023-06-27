use crossbeam_channel::Sender;
use crossbeam_channel::{select, unbounded};
use futures::executor::block_on;
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Empty, Summary};
use std::env;
use std::thread;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

pub mod book;
use crate::book::Book;

pub mod binance;
use crate::binance::BinanceClient;

pub mod bitstamp;
use crate::bitstamp::BitstampClient;

pub mod testmarket;
use crate::testmarket::GenClient;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[derive(Debug)]
struct MyOrderbookAggregator {
    clients_tx: Sender<tokio::sync::mpsc::Sender<Result<Summary, Status>>>,
}

#[tonic::async_trait]
impl OrderbookAggregator for MyOrderbookAggregator {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.clients_tx.send(tx).unwrap();
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line
    let pair = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("launch with currency pair"));

    // create queues
    let (orders_tx, orders_rx) = unbounded();
    let (clients_tx, clients_rx) = unbounded();

    // create grpc service
    let aggregator = MyOrderbookAggregator {
        clients_tx: clients_tx.clone(),
    };

    // main event loop
    let mut book = Book::new();
    let mut clients = Vec::<tokio::sync::mpsc::Sender<Result<Summary, Status>>>::new();
    thread::spawn(move || loop {
        select! {
            recv(orders_rx) -> orders => {
                book.add_orders(orders.unwrap());
                let summary = book.to_summary();
                clients.retain_mut(|client|
                     block_on(client.send(Ok(summary.clone()))).is_ok()
                );

            }
            recv(clients_rx) -> client => {
                let summary = book.to_summary();
                let uc = client.unwrap();
                if block_on(uc.send(Ok(summary))).is_ok() {
                    clients.push(uc);
                }
            }
        }
    });

    // binance client setup and wiring
    let binance_client = BinanceClient::new(pair.to_string());
    binance_client.do_main_loop(orders_tx.clone());

    // bitstamp client setup and wiring
    let bitstamp_client = BitstampClient::new(pair.to_string());
    bitstamp_client.do_main_loop(orders_tx.clone());

    //  test market
    let gen_client = GenClient::new(pair.to_string());
    gen_client.do_main_loop(orders_tx.clone());

    // setup address for grpc server binding
    let addr = "[::1]:1079".parse()?;

    // run grpc server
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;

    Ok(())
}
