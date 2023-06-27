/**
 *  Copyright (c) 2023 Antonino Nolano. Licensed under the MIT license, as
 * follows:
 *
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to
 * deal in the Software without restriction, including without limitation the
 * rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
 * sell copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE.
 */
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
        self.asks.retain(|Reverse(e)| e.exchange != orders.exchange);
        self.bids.retain(|e| e.exchange != orders.exchange);

        // insert orders
        for x in orders.asks.drain(..) {
            self.asks.push(Reverse(x)); // asc
        }
        for x in orders.bids.drain(..) {
            self.bids.push(x); // desc
        }

        // create summary
        self.summary.bids.clear();
        self.summary.asks.clear();

        // up top 10 asks
        let mut a = self.asks.clone();
        for _ in 1..11 {
            if let Some(Reverse(val)) = a.pop() {
                self.summary.asks.push(Level {
                    exchange: val.exchange.to_string(),
                    price: val.price.to_f64().unwrap(),
                    amount: val.amount.to_f64().unwrap(),
                });
            } else {
                break;
            }
        }

        // up to 10 bids
        let mut b = self.bids.clone();
        for _ in 1..11 {
            if let Some(val) = b.pop() {
                self.summary.bids.push(Level {
                    exchange: val.exchange.to_string(),
                    price: val.price.to_f64().unwrap(),
                    amount: val.amount.to_f64().unwrap(),
                });
            } else {
                break;
            }
        }

        // calculate spread
        match (self.asks.peek(), self.bids.peek()) {
            (Some(Reverse(ask)), Some(bid)) => {
                self.summary.spread = ask.price.to_f64().unwrap() - bid.price.to_f64().unwrap()
            }
            (None, Some(bid)) => self.summary.spread = bid.price.to_f64().unwrap() * -1.0,
            (Some(Reverse(ask)), None) => self.summary.spread = ask.price.to_f64().unwrap(),
            (None, None) => self.summary.spread = 0.0,
        }
    }

    pub fn to_summary(&self) -> Summary {
        self.summary.clone()
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
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

#[derive(Debug, PartialEq, Clone, Eq)]
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
