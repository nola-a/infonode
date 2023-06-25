use crate::book::Update;
use crate::book::Exchange;

pub struct BinanceClient {
    pair: String,
    endpoint: String
}

impl BinanceClient {

    pub fn new(pair: String) -> BinanceClient {
        // TODO
        BinanceClient{pair: pair.to_string(), endpoint: "endpoint".to_string()}
    }

    pub fn connect(&self) {
        // TODO
    }

    pub fn do_main_loop<F>(&self, f: F) where
        F: Fn(Update) {
        // TODO
        let orders = Update::new(Exchange::Binance);
        f(orders);
    }

}
