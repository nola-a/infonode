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
use crate::book::Exchange;
use crate::book::Update;
use crossbeam_channel::Sender;
use tungstenite::connect;
use url::Url;

pub struct BinanceClient {
    pair: String,
}

impl BinanceClient {
    pub fn new(pair: String) -> BinanceClient {
        BinanceClient {
            pair: pair.to_string(),
        }
    }

    pub fn do_main_loop(&self, tx: Sender<Update>) {
        let url = format!(
            "{}{}{}",
            "wss://stream.binance.com:9443/ws/", self.pair, "@depth20@100ms"
        );
        tokio::spawn(async move {
            let (mut socket, _) = connect(Url::parse(&url).unwrap()).expect("Can't connect");
            loop {
                let msg = socket.read_message().expect("Error reading message");
                let parsed = json::parse(&msg.to_string()).unwrap();
                let mut orders = Update::new(Exchange::Binance);
                if parsed.has_key("asks") && parsed["asks"].is_array() {
                    for i in 0..parsed["asks"].len() {
                        if parsed["asks"][i].len() == 2 {
                            orders.add_ask(
                                &parsed["asks"][i][0].to_string(),
                                &parsed["asks"][i][1].to_string(),
                                Exchange::Binance,
                            );
                        }
                    }
                }
                if parsed.has_key("bids") && parsed["bids"].is_array() {
                    for i in 0..parsed["bids"].len() {
                        if parsed["bids"][i].len() == 2 {
                            orders.add_bid(
                                &parsed["bids"][i][0].to_string(),
                                &parsed["bids"][i][1].to_string(),
                                Exchange::Binance,
                            );
                        }
                    }
                }
                tx.send(orders).unwrap();
            }
        });
    }
}
