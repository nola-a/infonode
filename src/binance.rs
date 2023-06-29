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
use log::info;
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

    pub async fn precisions(pair: String) -> (u64, u64) {
        let precision_url = format!(
            "{}{}",
            "https://api.binance.com/api/v3/exchangeInfo?symbol=",
            pair.to_ascii_uppercase()
        );

        let body = reqwest::get(precision_url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let parsed = json::parse(body.as_str()).unwrap();

        if parsed.has_key("symbols") && parsed["symbols"].is_array() && parsed["symbols"].len() == 1
        {
            return (
                parsed["symbols"][0]["quotePrecision"].as_u64().unwrap(),
                parsed["symbols"][0]["baseAssetPrecision"].as_u64().unwrap(),
            );
        }
        panic!("cannot get precisions from binance");
    }

    pub fn do_main_loop(&self, tx: Sender<Update>) {
        let stream_url = format!(
            "{}{}{}",
            "wss://stream.binance.com:9443/ws/", self.pair, "@depth10@100ms"
        );

        let pair = self.pair.clone();

        tokio::spawn(async move {
            let (p_prec, a_prec) = BinanceClient::precisions(pair).await;
            info!("precisions price={} amount={}", p_prec, a_prec);
            let (mut socket, _) = connect(Url::parse(&stream_url).unwrap()).expect("Can't connect");
            info!("websocket connected");
            loop {
                let msg = socket.read_message().expect("Error reading message");
                let parsed = json::parse(&msg.to_string()).unwrap();
                let mut orders = Update::new(Exchange::Binance, p_prec, a_prec);
                if parsed.has_key("asks") && parsed["asks"].is_array() {
                    for i in 0..parsed["asks"].len() {
                        if parsed["asks"][i].len() == 2 {
                            orders.add_ask(
                                &parsed["asks"][i][0].to_string(),
                                &parsed["asks"][i][1].to_string(),
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
                            );
                        }
                    }
                }
                tx.send(orders).unwrap();
            }
        });
    }
}
