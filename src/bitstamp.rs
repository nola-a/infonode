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
use tungstenite::{connect, Message};
use url::Url;

pub struct BitstampClient {
    pair: String,
}

impl BitstampClient {
    pub fn new(pair: String) -> BitstampClient {
        BitstampClient {
            pair: pair.to_string(),
        }
    }

    async fn precisions(pair: String) -> (u64, u64) {
        let body = reqwest::get("https://www.bitstamp.net/api/v2/trading-pairs-info")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let parsed = json::parse(body.as_str()).unwrap();

        if parsed.is_array() {
            for i in 0..parsed.len() {
                if parsed[i].has_key("url_symbol") && parsed[i]["url_symbol"].to_string() == pair {
                    return (
                        parsed[i]["base_decimals"].as_u64().unwrap(),
                        parsed[i]["instant_order_counter_decimals"]
                            .as_u64()
                            .unwrap(),
                    );
                }
            }
        }

        panic!("cannot get precisions from bitstamp");
    }

    pub fn do_main_loop(&self, tx: Sender<Update>) {
        let stream_url = "wss://ws.bitstamp.net";
        let submessage = format!(
            "{}{}{}",
            r#"{"event":"bts:subscribe","data":{"channel":"order_book_"#, self.pair, "\"}}"
        );

        let pair = self.pair.clone();

        tokio::spawn(async move {
            let (p_prec, a_prec) = BitstampClient::precisions(pair).await;
            let (mut socket, _) = connect(Url::parse(&stream_url).unwrap()).expect("Can't connect");
            socket
                .write_message(Message::Text(submessage.into()))
                .unwrap();
            let msg = socket.read_message().expect("Error reading message");
            let parsed = json::parse(&msg.to_string()).unwrap();
            if parsed.has_key("event") && parsed["event"] == "bts:subscription_succeeded" {
                loop {
                    let msg = socket.read_message().expect("Error reading message");
                    let parsed = json::parse(&msg.to_string()).unwrap();
                    let mut orders = Update::new(Exchange::Bitstamp, p_prec, a_prec);
                    if parsed.has_key("data")
                        && parsed["data"].has_key("asks")
                        && parsed["data"]["asks"].is_array()
                    {
                        for i in 0..parsed["data"]["asks"].len() {
                            if parsed["data"]["asks"][i].len() == 2 {
                                orders.add_ask(
                                    &parsed["data"]["asks"][i][0].to_string(),
                                    &parsed["data"]["asks"][i][1].to_string(),
                                );
                            }
                        }
                    }
                    if parsed.has_key("data")
                        && parsed["data"].has_key("bids")
                        && parsed["data"]["bids"].is_array()
                    {
                        for i in 0..parsed["data"]["bids"].len() {
                            if parsed["data"]["bids"][i].len() == 2 {
                                orders.add_bid(
                                    &parsed["data"]["bids"][i][0].to_string(),
                                    &parsed["data"]["bids"][i][1].to_string(),
                                );
                            }
                        }
                    }
                    tx.send(orders).unwrap();
                }
            }
        });
    }
}
