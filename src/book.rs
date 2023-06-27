use crate::orderbook::{Level, Summary};
use bigdecimal::{BigDecimal, ToPrimitive};
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::str::FromStr;

pub struct Book {
    asks: BinaryHeap<Reverse<Entry>>,
    bids: BinaryHeap<Entry>,
    summary: Summary,
}

impl Book {
    pub fn new() -> Book {
        Book {
            asks: BinaryHeap::new(),
            bids: BinaryHeap::new(),
            summary: Summary::default(),
        }
    }

    pub fn add_orders(&mut self, mut orders: Update) -> () {
        // remove older orders
        self.asks.retain(|e| e.0.exchange != orders.exchange);
        self.bids.retain(|e| e.exchange != orders.exchange);

        // insert orders
        for x in orders.asks.drain(..) {
            self.asks.push(Reverse(x)); // asc
        }
        for x in orders.bids.drain(..) {
            self.bids.push(x); // desc
        }

        // create summary
        let mut bids: Vec<Level> = Vec::new();
        let mut asks: Vec<Level> = Vec::new();

        // top 10 asks
        let mut a = self.asks.clone();
        for _ in 1..11 {
            let val: Entry = a.pop().unwrap().0;
            asks.push(Level {
                exchange: val.exchange.to_string(),
                price: val.price.to_f64().unwrap(),
                amount: val.price.to_f64().unwrap(),
            });
        }

        // top 10 bids
        let mut b = self.bids.clone();
        for _ in 1..11 {
            let val: Entry = b.pop().unwrap();
            bids.push(Level {
                exchange: val.exchange.to_string(),
                price: val.price.to_f64().unwrap(),
                amount: val.price.to_f64().unwrap(),
            });
        }

        self.summary.asks = asks;
        self.summary.bids = bids;
        self.summary.spread = self.asks.peek().unwrap().0.price.to_f64().unwrap()
            - self.bids.peek().unwrap().price.to_f64().unwrap();
    }

    pub fn to_summary(&self) -> Summary {
        self.summary.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Exchange {
    Binance,
    Bitstamp,
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Exchange::Bitstamp => write!(f, "bitstamp"),
            Exchange::Binance => write!(f, "binance"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Entry {
    price: BigDecimal,
    amount: BigDecimal,
    exchange: Exchange,
}

#[derive(Debug)]
pub struct Update {
    exchange: Exchange,
    bids: Vec<Entry>,
    asks: Vec<Entry>,
}

impl Update {
    pub fn new(e: Exchange) -> Update {
        Update {
            exchange: e,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn add_bid(&mut self, price: &str, amount: &str, e: Exchange) {
        self.bids.push(Entry {
            price: BigDecimal::from_str(price).unwrap(),
            amount: BigDecimal::from_str(amount).unwrap(),
            exchange: e,
        });
    }

    pub fn add_ask(&mut self, price: &str, amount: &str, e: Exchange) {
        self.asks.push(Entry {
            price: BigDecimal::from_str(price).unwrap(),
            amount: BigDecimal::from_str(amount).unwrap(),
            exchange: e,
        });
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.price.partial_cmp(&self.price)
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Entry) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
