use crate::book::Update;
use crate::book::Exchange;

pub struct BitstampClient {
    pair: String,
    endpoint: String
}

impl BitstampClient {

    pub fn new(pair: String) -> BitstampClient {
        // TODO
        BitstampClient{pair: pair.to_string(), endpoint: "endpoint".to_string()}
    }

    pub fn connect(&self) {
        // TODO
    }

    pub fn do_main_loop<F>(&self, f: F) where
         F: Fn(Update) {
         // TODO
         let orders = Update::new(Exchange::Bitstamp);
         f(orders);
    }

}