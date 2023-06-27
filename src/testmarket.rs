use crate::book::Exchange;
use crate::book::Update;
use crossbeam_channel::Sender;
use tokio::time::{sleep, Duration};

pub struct GenClient {
    pair: String,
}

impl GenClient {
    pub fn new(pair: String) -> GenClient {
        GenClient {
            pair: pair.to_string(),
        }
    }

    pub fn do_main_loop(&self, tx: Sender<Update>) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(1000)).await;
                let mut orders = Update::new(Exchange::Binance);
                orders.add_ask("201223.45", "122.44", Exchange::Binance);
                orders.add_ask("211223.45", "122.44", Exchange::Binance);
                orders.add_ask("221223.45", "122.44", Exchange::Binance);
                orders.add_ask("231223.45", "122.44", Exchange::Binance);
                orders.add_ask("241223.45", "122.44", Exchange::Binance);
                orders.add_ask("251223.45", "122.44", Exchange::Binance);
                orders.add_ask("261223.45", "122.44", Exchange::Binance);
                orders.add_ask("271223.45", "122.44", Exchange::Binance);
                orders.add_ask("281223.45", "122.44", Exchange::Binance);
                orders.add_ask("291223.45", "122.44", Exchange::Binance);
                orders.add_ask("301223.45", "122.44", Exchange::Binance);
                orders.add_ask("311223.45", "122.44", Exchange::Binance);
                orders.add_bid("11223.45", "122.44", Exchange::Binance);
                orders.add_bid("21223.45", "122.44", Exchange::Binance);
                orders.add_bid("31223.45", "122.44", Exchange::Binance);
                orders.add_bid("41223.45", "122.44", Exchange::Binance);
                orders.add_bid("51223.45", "122.44", Exchange::Binance);
                orders.add_bid("61223.45", "122.44", Exchange::Binance);
                orders.add_bid("71223.45", "122.44", Exchange::Binance);
                orders.add_bid("81223.45", "122.44", Exchange::Binance);
                orders.add_bid("91223.45", "122.44", Exchange::Binance);
                orders.add_bid("101223.45", "122.44", Exchange::Binance);
                orders.add_bid("111223.45", "122.44", Exchange::Binance);
                tx.send(orders).unwrap();

                sleep(Duration::from_millis(1000)).await;
                let mut orders = Update::new(Exchange::Bitstamp);
                orders.add_ask("201223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("211223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("221223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("231223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("241223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("251223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("261223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("271223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("281223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("291223.45", "122.44", Exchange::Bitstamp);
                orders.add_ask("301223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("11223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("21223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("31223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("41223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("51223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("61223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("71223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("81223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("91223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("101223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("111223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("121223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("131223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("141223.45", "122.44", Exchange::Bitstamp);
                orders.add_bid("151223.45", "122.44", Exchange::Bitstamp);
                tx.send(orders).unwrap();
            }
        });
    }
}
