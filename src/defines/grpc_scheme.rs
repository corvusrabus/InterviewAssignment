use rust_decimal::Decimal;
use smallvec::SmallVec;

#[derive(Debug)]
pub struct OrderbookUpdateMessage<'a> {
    pub spread: Decimal,
    pub asks: Vec<MessageBookLevel<'a>>,
    pub bids: Vec<MessageBookLevel<'a>>,
}

#[derive(Debug)]
pub struct MessageBookLevel<'a> {
    pub(crate) exchange: &'a str,
    pub(crate) price: Decimal,
    pub(crate) amount: Decimal,
}

tonic::include_proto!("orderbook");
