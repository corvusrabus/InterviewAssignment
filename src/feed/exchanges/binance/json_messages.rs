use serde::{Serialize, Deserialize};
use smallvec::SmallVec;
use crate::feed::exchanges::binance::BINANCE_BOOK_DEPTH;
use crate::marketdata::{Orderbook, BookLevel};

#[derive(Serialize)]
pub struct BookSubRequest {
    method: String,
    params: Vec<String>,
    id: usize,
}

impl BookSubRequest {
    const FIRST_ID: usize = 1;
    pub fn new(symbol: &str) -> Self {
        Self { method: String::from("SUBSCRIBE"), params: vec![format!("{symbol}@depth{BINANCE_BOOK_DEPTH}@100ms")], id: Self::FIRST_ID }
    }
}


#[derive(Deserialize)]
pub struct BookSubRequestResponse {
    pub result: Option<()>,
    pub id: usize,
}
#[derive(Deserialize)]
pub(crate) struct BinanceBookMessage{
    pub data : BinanceBookMessageData,
}


#[derive(Deserialize)]
pub(crate) struct BinanceBookMessageData{
    pub bids : SmallVec<[BookLevel;BINANCE_BOOK_DEPTH]>,
    pub asks : SmallVec<[BookLevel;BINANCE_BOOK_DEPTH]>,
}
