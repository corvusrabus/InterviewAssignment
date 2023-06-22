use std::cmp::{max, Ordering, Reverse};
use std::collections::BinaryHeap;
use halfbrown::HashMap;
use rust_decimal::Decimal;
use smallvec::SmallVec;
use crate::defines::{BOOK_LEVELS_USED, Exchange};
use serde::Deserialize;
use crate::defines::grpc_scheme::{MessageBookLevel, OrderbookUpdateMessage};
use crate::helper::{is_ascending_by_key, is_descending_by_key};

mod books;


#[derive(Debug, Copy, Clone, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct BookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}


#[derive(Debug, Clone)]
pub(crate) struct Orderbook {
    bids: SmallVec<[BookLevel; 10]>,
    asks: SmallVec<[BookLevel; 10]>,
}

impl Orderbook {
    pub fn new(bids: SmallVec<[BookLevel; 10]>, asks: SmallVec<[BookLevel; 10]>) -> Self {
        debug_assert!(is_descending_by_key(bids.as_slice(), |level| level.price));
        debug_assert!(is_ascending_by_key(asks.as_slice(), |level| level.price));
        Self { bids, asks }
    }
}


pub(crate) struct BookAggregator {
    books: HashMap<Exchange, Orderbook>,
}

impl BookAggregator {
    pub fn add_new_book(&mut self, exchange: Exchange, book: Orderbook) {
        self.books.insert(exchange, book);
    }
    pub fn make_grpc_message(&self) -> OrderbookUpdateMessage<'static> {
        #[derive(PartialEq)]
        #[derive(Eq)]
        struct BookLevelAndExchangeHelper<'a>(&'a BookLevel, Exchange);
        impl<'a> PartialOrd for BookLevelAndExchangeHelper<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }
        impl<'a> Ord for BookLevelAndExchangeHelper<'a> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.0.cmp(&other.0)
            }
        }
        let expected_number_of_entries = BOOK_LEVELS_USED * self.books.len();
        let mut bids = Vec::with_capacity(expected_number_of_entries);
        let mut asks = Vec::with_capacity(expected_number_of_entries);
        let mut bid_indices: HashMap<Exchange, usize> = HashMap::new();
        let mut ask_indices: HashMap<Exchange, usize> = HashMap::new();
        // Top is highest bid aggregated over all exchanges
        let mut bid_heap = BinaryHeap::new();
        // Top is lowest ask aggregated over all exchanges
        let mut ask_heap = BinaryHeap::new();
        for (exchange, book) in self.books.iter() {
            bid_indices.insert(*exchange, 1);
            ask_indices.insert(*exchange, 1);
            if let Some(level) = book.bids.first()
            {
                bid_heap.push(BookLevelAndExchangeHelper(level, *exchange));
            }
            if let Some(level) = book.asks.first()
            {
                ask_heap.push(Reverse(BookLevelAndExchangeHelper(level, *exchange)));
            }
        }
        for i in 0..BOOK_LEVELS_USED {
            if let Some(BookLevelAndExchangeHelper(level, exchange)) = bid_heap.pop() {
                let index = bid_indices.remove(&exchange).unwrap();
                if let Some(bid_level) = self.books.get(&exchange).unwrap().bids.get(index) {
                    bid_heap.push(BookLevelAndExchangeHelper(level, exchange));
                    bids.push(MessageBookLevel {
                        exchange: exchange.name(),
                        price: bid_level.price,
                        amount: bid_level.quantity,
                    });
                    bid_indices.insert_nocheck(exchange, index + 1);
                }
            }
            if let Some(Reverse(BookLevelAndExchangeHelper(level, exchange))) = ask_heap.pop() {
                let index = ask_indices.remove(&exchange).unwrap();
                if let Some(ask_level) = self.books.get(&exchange).unwrap().asks.get(index) {
                    ask_heap.push(Reverse(BookLevelAndExchangeHelper(level, exchange)));
                    asks.push(MessageBookLevel {
                        exchange: exchange.name(),
                        price: ask_level.price,
                        amount: ask_level.quantity,
                    });
                    ask_indices.insert_nocheck(exchange, index + 1);
                }
            }
        }
       let spread = if asks.first().is_some() && bids.first().is_some() {
           asks.first().as_ref().unwrap().price -  bids.first().as_ref().unwrap().price
       }
       else {
           Decimal::ZERO
       };
        OrderbookUpdateMessage { spread, asks, bids }
    }
    pub fn new() -> Self {
        Self { books: HashMap::new() }
    }
}

#[cfg(test)]
mod test {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use crate::marketdata::BookLevel;
    use rand::Rng;

    // ensure that PartialOrd orders book levels by price first
    #[test]
    fn book_levels_ordered_by_price() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let a: Decimal = rng.gen();
            let b: Decimal = rng.gen();
            let c: Decimal = rng.gen();
            let d: Decimal = rng.gen();
            if a == b {
                continue;
            }
            assert_eq!(a <= b, BookLevel { price: a, quantity: c } <= BookLevel { price: b, quantity: d })
        }
    }
}