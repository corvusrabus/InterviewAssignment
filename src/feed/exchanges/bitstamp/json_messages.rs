use crate::marketdata::{BookLevel, Orderbook};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Serialize)]
pub struct BookSubRequest {
    event: String,
    data: BookSubRequestData,
}
#[derive(Serialize)]
struct BookSubRequestData {
    pub channel: String,
}
impl BookSubRequest {
    pub fn new(symbol: &str) -> Self {
        Self {
            event: "bts:subscribe".to_string(),
            data: BookSubRequestData {
                channel: format!("order_book_{symbol}"),
            },
        }
    }
}

#[derive(Deserialize)]
pub struct BookSubRequestResponse {
    pub event: String,
    pub channel: String,
}
#[derive(Deserialize)]
pub(crate) struct BookMessage {
    pub data: BookMessageData,
}

#[derive(Deserialize)]
pub(crate) struct BookMessageData {
    pub bids: SmallVec<[BookLevel; 128]>,
    pub asks: SmallVec<[BookLevel; 128]>,
}
