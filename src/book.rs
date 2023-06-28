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
use log::{debug, trace};
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Sub;
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
        // remove existing orders orders.exchange
        debug!("remove {} orders", orders.exchange);

        self.asks.retain(|Reverse(e)| e.exchange != orders.exchange);
        self.bids.retain(|e| e.exchange != orders.exchange);

        // insert orders
        debug!(
            "insert {} ask orders from {}",
            orders.asks.len(),
            orders.exchange
        );
        for x in orders.asks.drain(..) {
            self.asks.push(Reverse(x)); // asc
        }
        debug!(
            "insert {} bids orders from {}",
            orders.bids.len(),
            orders.exchange
        );
        for x in orders.bids.drain(..) {
            self.bids.push(x); // desc
        }

        // create summary
        self.summary.bids.clear();
        self.summary.asks.clear();

        // up top 20 asks
        let mut a = self.asks.clone();
        for _ in 1..21 {
            if let Some(Reverse(val)) = a.pop() {
                self.summary.asks.insert(
                    0,
                    Level {
                        exchange: val.exchange.to_string(),
                        price: val.price.to_f64().unwrap(),
                        amount: val.amount.to_f64().unwrap(),
                    },
                );
            } else {
                break;
            }
        }

        // up to 20 bids
        let mut b = self.bids.clone();
        for _ in 1..21 {
            if let Some(val) = b.pop() {
                self.summary.bids.insert(
                    0,
                    Level {
                        exchange: val.exchange.to_string(),
                        price: val.price.to_f64().unwrap(),
                        amount: val.amount.to_f64().unwrap(),
                    },
                );
            } else {
                break;
            }
        }

        // calculate spread
        match (self.asks.peek(), self.bids.peek()) {
            (Some(Reverse(ask)), Some(bid)) => {
                self.summary.spread = ask
                    .price
                    .clone()
                    .sub(bid.price.clone())
                    .with_prec(orders.price_prec)
                    .to_f64()
                    .unwrap();
            }
            (None, Some(bid)) => self.summary.spread = bid.price.to_f64().unwrap() * (-1.0),
            (Some(Reverse(ask)), None) => self.summary.spread = ask.price.to_f64().unwrap(),
            (None, None) => self.summary.spread = 0.0,
        }

        debug!(
            "summary.spread={} top {} asks top {} bids",
            self.summary.spread,
            self.summary.asks.len(),
            self.summary.bids.len()
        );
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
    price_prec: u64,
    amount_prec: u64,
}

impl Update {
    pub fn new(e: Exchange, price_prec: u64, amount_prec: u64) -> Update {
        Update {
            exchange: e,
            bids: Vec::new(),
            asks: Vec::new(),
            price_prec: price_prec,
            amount_prec: amount_prec,
        }
    }

    pub fn add_bid(&mut self, price: &str, amount: &str) {
        self.bids.push(Entry {
            price: BigDecimal::from_str(price)
                .unwrap()
                .with_prec(self.price_prec),
            amount: BigDecimal::from_str(amount)
                .unwrap()
                .with_prec(self.amount_prec),
            exchange: self.exchange.clone(),
        });
    }

    pub fn add_ask(&mut self, price: &str, amount: &str) {
        self.asks.push(Entry {
            price: BigDecimal::from_str(price)
                .unwrap()
                .with_prec(self.price_prec),
            amount: BigDecimal::from_str(amount)
                .unwrap()
                .with_prec(self.amount_prec),
            exchange: self.exchange.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spread_1() {
        let mut orders = Update::new(Exchange::Binance, 5, 5);
        orders.add_ask("0.00555", "1234");
        orders.add_bid("0.00551", "1234");
        let mut book = Book::new();
        book.add_orders(orders);
        assert_eq!(book.summary.spread, 0.00004);
    }

    #[test]
    fn test_spread_2() {
        let mut orders = Update::new(Exchange::Binance, 10, 10);
        orders.add_ask("0.00555", "1234");
        let mut book = Book::new();
        book.add_orders(orders);
        assert_eq!(book.summary.spread, 0.00555);
    }

    #[test]
    fn test_spread_3() {
        let mut orders = Update::new(Exchange::Binance, 10, 10);
        orders.add_bid("0.00555", "1234");
        let mut book = Book::new();
        book.add_orders(orders);
        assert_eq!(book.summary.spread, -0.00555);
    }

    #[test]
    fn test_top_bid() {
        let mut orders = Update::new(Exchange::Binance, 10, 10);
        orders.add_bid("1", "1");
        orders.add_bid("3", "3");
        orders.add_bid("2", "2");
        orders.add_bid("6", "6");

        let mut book = Book::new();
        book.add_orders(orders);

        assert_eq!(
            book.summary.bids[0],
            Level {
                price: 6.0,
                amount: 6.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[1],
            Level {
                price: 3.0,
                amount: 3.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[2],
            Level {
                price: 2.0,
                amount: 2.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[3],
            Level {
                price: 1.0,
                amount: 1.0,
                exchange: "binance".to_string()
            }
        );
    }

    #[test]
    fn test_top_bid_2() {
        let mut orders = Update::new(Exchange::Binance, 10, 10);
        orders.add_bid("0.000010001", "1");
        orders.add_bid("0.030003", "3");
        orders.add_bid("0.0020002", "2");
        orders.add_bid("0.06", "6");

        let mut book = Book::new();
        book.add_orders(orders);

        assert_eq!(
            book.summary.bids[0],
            Level {
                price: 0.06_f64,
                amount: 6.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[1],
            Level {
                price: 0.030003_f64,
                amount: 3.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[2],
            Level {
                price: 0.0020002_f64,
                amount: 2.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[3],
            Level {
                price: 0.000010001_f64,
                amount: 1.0,
                exchange: "binance".to_string()
            }
        );
    }

    #[test]
    fn test_top_ask() {
        let mut orders = Update::new(Exchange::Binance, 10, 10);
        orders.add_ask("1", "1");
        orders.add_ask("3", "3");
        orders.add_ask("2", "2");
        orders.add_ask("6", "6");

        let mut book = Book::new();
        book.add_orders(orders);

        assert_eq!(
            book.summary.asks[0],
            Level {
                price: 1.0,
                amount: 1.0,
                exchange: "binance".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[1],
            Level {
                price: 2.0,
                amount: 2.0,
                exchange: "binance".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[2],
            Level {
                price: 3.0,
                amount: 3.0,
                exchange: "binance".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[3],
            Level {
                price: 6.0,
                amount: 6.0,
                exchange: "binance".to_string()
            }
        );
    }

    #[test]
    fn test_decimal() {
        let e = BigDecimal::from_str("0.00000030003");
        assert_eq!(
            0.00000030003_f64,
            e.unwrap().with_prec(11).to_f64().unwrap()
        );
    }

    #[test]
    fn test_top_ask_exchanges() {
        let mut orders1 = Update::new(Exchange::Binance, 2, 2);
        orders1.add_ask("1", "1");
        orders1.add_ask("3", "3");
        orders1.add_ask("2", "2");
        orders1.add_ask("6", "6");

        let mut orders2 = Update::new(Exchange::Bitstamp, 2, 2);
        orders2.add_ask("1.1", "1");
        orders2.add_ask("3.1", "3");
        orders2.add_ask("2.1", "2");
        orders2.add_ask("6.1", "6");

        let mut book = Book::new();
        book.add_orders(orders1);
        book.add_orders(orders2);

        assert_eq!(
            book.summary.asks[0],
            Level {
                price: 1.0,
                amount: 1.0,
                exchange: "binance".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[1],
            Level {
                price: 1.1,
                amount: 1.0,
                exchange: "bitstamp".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[2],
            Level {
                price: 2.0,
                amount: 2.0,
                exchange: "binance".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[3],
            Level {
                price: 2.1,
                amount: 2.0,
                exchange: "bitstamp".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[4],
            Level {
                price: 3.0,
                amount: 3.0,
                exchange: "binance".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[5],
            Level {
                price: 3.1,
                amount: 3.0,
                exchange: "bitstamp".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[6],
            Level {
                price: 6.0,
                amount: 6.0,
                exchange: "binance".to_string()
            }
        );
    }
}
