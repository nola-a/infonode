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
        // remove existing orders for orders.exchange
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
                self.summary.asks.insert(0, Level {
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
                self.summary.bids.insert(0, Level {
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
                self.summary.spread = ask.price.clone().sub(bid.price.clone()).to_f64().unwrap();
            }
            (None, Some(bid)) => self.summary.spread = bid.price.to_f64().unwrap() * (-1.0),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spread_1() {
        let mut orders = Update::new(Exchange::Binance);
        orders.add_ask("0.00555", "1234", Exchange::Binance);
        orders.add_bid("0.00551", "1234", Exchange::Binance);
        let mut book = Book::new();
        book.add_orders(orders);
        assert_eq!(book.summary.spread, 0.00004);
    }

    #[test]
    fn test_spread_2() {
        let mut orders = Update::new(Exchange::Binance);
        orders.add_ask("0.00555", "1234", Exchange::Binance);
        let mut book = Book::new();
        book.add_orders(orders);
        assert_eq!(book.summary.spread, 0.00555);
    }

    #[test]
    fn test_spread_3() {
        let mut orders = Update::new(Exchange::Binance);
        orders.add_bid("0.00555", "1234", Exchange::Binance);
        let mut book = Book::new();
        book.add_orders(orders);
        assert_eq!(book.summary.spread, -0.00555);
    }

    #[test]
    fn test_top_bid() {
        let mut orders = Update::new(Exchange::Binance);
        orders.add_bid("1", "1", Exchange::Binance);
        orders.add_bid("3", "3", Exchange::Bitstamp);
        orders.add_bid("2", "2", Exchange::Binance);
        orders.add_bid("6", "6", Exchange::Bitstamp);

        let mut book = Book::new();
        book.add_orders(orders);

        assert_eq!(
            book.summary.bids[0],
            Level {
                price: 6.0,
                amount: 6.0,
                exchange: "bitstamp".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[1],
            Level {
                price: 3.0,
                amount: 3.0,
                exchange: "bitstamp".to_string()
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
        let mut orders = Update::new(Exchange::Binance);
        orders.add_bid("0.00000000010001", "1", Exchange::Binance);
        orders.add_bid("0.00000030003", "3", Exchange::Bitstamp);
        orders.add_bid("0.000000020002", "2", Exchange::Binance);
        orders.add_bid("0.0060006", "6", Exchange::Bitstamp);

        let mut book = Book::new();
        book.add_orders(orders);

        assert_eq!(
            book.summary.bids[0],
            Level {
                price: 0.0060006_f64,
                amount: 6.0,
                exchange: "bitstamp".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[1],
            Level {
                price: 0.00000030003_f64,
                amount: 3.0,
                exchange: "bitstamp".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[2],
            Level {
                price: 0.000000020002_f64,
                amount: 2.0,
                exchange: "binance".to_string()
            }
        );
        assert_eq!(
            book.summary.bids[3],
            Level {
                price: 0.00000000010001_f64,
                amount: 1.0,
                exchange: "binance".to_string()
            }
        );
    }

    #[test]
    fn test_top_ask() {
        let mut orders = Update::new(Exchange::Binance);
        orders.add_ask("1", "1", Exchange::Binance);
        orders.add_ask("3", "3", Exchange::Bitstamp);
        orders.add_ask("2", "2", Exchange::Binance);
        orders.add_ask("6", "6", Exchange::Bitstamp);

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
                exchange: "bitstamp".to_string()
            }
        );

        assert_eq!(
            book.summary.asks[3],
            Level {
                price: 6.0,
                amount: 6.0,
                exchange: "bitstamp".to_string()
            }
        );
    }

    #[test]
    fn test_decimal() {
        let e = BigDecimal::from_str("0.00000030003");
        assert_eq!(0.00000030003_f64, e.unwrap().to_f64().unwrap());
    }
}
