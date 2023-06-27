use crate::book::Exchange;
use crate::book::Update;
use crossbeam_channel::Sender;
use tokio::time::{sleep, Duration};

pub struct BinanceClient {
    pair: String,
}

//{
//    "lastUpdateId": 160,  // Last update ID
//    "bids": [             // Bids to be updated
//      [
//        "0.0024",         // Price level to be updated
//        "10"              // Quantity
//      ]
//    ],
//    "asks": [             // Asks to be updated
//      [
//        "0.0026",         // Price level to be updated
//        "100"             // Quantity
//      ]
//    ]
//  }

//let url = format!("{}{}{}", "wss://stream.binance.com:9443/ws/", self.pair, "@depth20@100ms");
//let url = format!("{}{}{}", "wss://testnet.binance.vision/ws", self.pair, "@depth20@100ms");

impl BinanceClient {
    pub fn new(pair: String) -> BinanceClient {
        // TODO
        BinanceClient {
            pair: pair.to_string(),
        }
    }

    pub fn do_main_loop(&self, tx: Sender<Update>) {}
}
