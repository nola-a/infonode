use crate::book::Exchange;
use crate::book::Update;
use crossbeam_channel::Sender;
use tokio::time::{sleep, Duration};

pub struct BitstampClient {
    pair: String,
}

impl BitstampClient {
    pub fn new(pair: String) -> BitstampClient {
        // TODO
        BitstampClient {
            pair: pair.to_string(),
        }
    }

    pub fn do_main_loop(&self, tx: Sender<Update>) {}
}
