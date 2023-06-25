use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Empty, Summary};
use std::thread;
use crossbeam_channel::{select, unbounded};
use crossbeam_channel::{Sender, Receiver};
use std::env;
use tokio::task;

pub mod book;
use crate::book::Book;

pub mod binance;
use crate::binance::BinanceClient;

pub mod bitstamp;
use crate::bitstamp::BitstampClient;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[derive(Debug)]
struct MyOrderbookAggregator {
    clients_tx: Sender<tokio::sync::mpsc::Sender<Result<Summary, Status>>>
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
    let args: Vec<String> = env::args().collect();
    let pair = &args[1];

    // create queues
    let (orders_tx, orders_rx) = unbounded();
    let (clients_tx, clients_rx) = unbounded();

    // create grpc service
    let aggregator = MyOrderbookAggregator{clients_tx: clients_tx.clone()};

    // binance client setup and wiring
    let binance_client = BinanceClient::new(pair.to_string());
    binance_client.connect();
    let tx1 = orders_tx.clone();
    thread::spawn(move || 
        binance_client.do_main_loop(|orders| tx1.send(orders).unwrap())
    );

    // bistamp client setup and wiring
    let bitstamp_client = BitstampClient::new(pair.to_string());
    bitstamp_client.connect();
    let tx2 = orders_tx.clone();
    thread::spawn(move || 
        bitstamp_client.do_main_loop(|orders| tx2.send(orders).unwrap())
    );

    // main event loop
    let book = Book::new();
    let mut clients = Vec::<tokio::sync::mpsc::Sender<Result<Summary, Status>>>::new();
    thread::spawn(move || 
        select! {
            recv(orders_rx) -> orders => { 
                book.add_orders(orders.unwrap());
                let summary = book.to_summary();
                for client in clients {
                    let s = summary.clone();
                    let c = client.clone();
                    task::spawn(async move {
                        c.send(Ok(s)).await.unwrap();
                    });
                }
            }
            recv(clients_rx) -> client => {
                let uc = client.unwrap();
                let c = uc.clone();
                clients.push(uc);
                let s = book.to_summary();
                task::spawn(async move {
                    c.send(Ok(s)).await.unwrap();
                });

            }
        }
    );

    // setup address for grpc server binding
    let addr = "[::1]:1079".parse()?;

    // run grpc server
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;

    Ok(())
}
