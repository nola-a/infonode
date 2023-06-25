use std::collections::BinaryHeap;
use crate::orderbook::{Empty, Summary};

pub struct Book {
    heap: BinaryHeap<i32>
}

impl Book {

    pub fn new() -> Book {
        Book {heap: BinaryHeap::new()}
    }

    pub fn add_orders(&self, orders: Update) -> () {
        // TODO
    }

    pub fn to_summary(&self) -> Summary {
        // TODO
        return Summary::default();
    }

}

pub enum Exchange {
    Binance,
    Bitstamp
}

pub struct Entry {
    price: f64,
    amount: f64 
}

pub struct Update {
    exchange: Exchange,
    bids: Vec<Entry>,
    asks: Vec<Entry>
}

impl Update {
    pub fn new(e: Exchange) -> Update {
        Update{exchange: e, bids: Vec::new(), asks: Vec::new()}
    }
}
