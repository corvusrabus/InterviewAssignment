use crate::defines::book_callback::BookCallback;
use crate::defines::grpc_scheme::{Level, Summary};
use crate::defines::{Exchange, BOOK_LEVELS_USED};
use crate::helper::{is_ascending_by_key, is_descending_by_key};
use async_broadcast::Sender;
use async_trait::async_trait;
use halfbrown::HashMap;
use log::error;
use parking_lot::Mutex;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use smallvec::SmallVec;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use tonic::Status;

#[derive(Debug, Copy, Clone, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct BookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Clone)]
pub(crate) struct Orderbook {
    bids: SmallVec<[BookLevel; BOOK_LEVELS_USED]>,
    asks: SmallVec<[BookLevel; BOOK_LEVELS_USED]>,
}

impl Orderbook {
    pub fn new(
        bids: SmallVec<[BookLevel; BOOK_LEVELS_USED]>,
        asks: SmallVec<[BookLevel; BOOK_LEVELS_USED]>,
    ) -> Self {
        debug_assert!(is_descending_by_key(bids.as_slice(), |level| level.price));
        debug_assert!(is_ascending_by_key(asks.as_slice(), |level| level.price));
        Self { bids, asks }
    }
}

pub(crate) struct BookAggregatorCallback {
    aggregator: Mutex<BookAggregator>,
    sender: Sender<Result<Summary, Status>>,
}

impl BookAggregatorCallback {
    pub fn new(sender: Sender<Result<Summary, Status>>) -> Self {
        Self {
            aggregator: Mutex::new(BookAggregator::new()),
            sender,
        }
    }
}

#[async_trait]
impl BookCallback for BookAggregatorCallback {
    async fn accept_book(&self, book: Orderbook, exchange: Exchange) {
        let message = {
            let mut locked = self.aggregator.lock();
            locked.add_new_book(book, exchange);
            locked.make_summary()
        };
        let send_result = self.sender.broadcast(Ok(message)).await;
        // Overflow should not happen but we should be able to see if it happens
        // Error can be ignored because that only means that currently no one is subscribed
        if let Ok(Some(_)) = send_result {
            error!(target : "BookAggregatorCallback", "broadcast channel overflowed");
        }
    }
}

#[derive(Debug)]
pub(crate) struct BookAggregator<const N_LEVELS: usize = BOOK_LEVELS_USED> {
    books: HashMap<Exchange, Orderbook>,
}

impl<const N_LEVELS: usize> BookAggregator<N_LEVELS> {
    pub fn add_new_book(&mut self, book: Orderbook, exchange: Exchange) {
        self.books.insert(exchange, book);
    }
    pub fn make_summary(&self) -> Summary {
        #[derive(PartialEq, Eq)]
        struct BookLevelAndExchangeHelper<'a>(&'a BookLevel, Exchange);
        impl<'a> PartialOrd for BookLevelAndExchangeHelper<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.0.partial_cmp(other.0)
            }
        }
        impl<'a> Ord for BookLevelAndExchangeHelper<'a> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.0.cmp(other.0)
            }
        }
        let expected_number_of_entries = N_LEVELS;
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
            if let Some(level) = book.bids.first() {
                bid_heap.push(BookLevelAndExchangeHelper(level, *exchange));
            }
            if let Some(level) = book.asks.first() {
                ask_heap.push(Reverse(BookLevelAndExchangeHelper(level, *exchange)));
            }
        }
        for _ in 0..N_LEVELS {
            if let Some(BookLevelAndExchangeHelper(level, exchange)) = bid_heap.pop() {
                bids.push(Level {
                    exchange: exchange.name().to_string(),
                    price: level.price.to_f64().unwrap_or_default(),
                    amount: level.quantity.to_f64().unwrap_or_default(),
                });
                let index = bid_indices.remove(&exchange).unwrap();
                if let Some(bid_level) = self.books.get(&exchange).unwrap().bids.get(index) {
                    bid_heap.push(BookLevelAndExchangeHelper(bid_level, exchange));
                    bid_indices.insert_nocheck(exchange, index + 1);
                }
            }
            if let Some(Reverse(BookLevelAndExchangeHelper(level, exchange))) = ask_heap.pop() {
                let index = ask_indices.remove(&exchange).unwrap();
                asks.push(Level {
                    exchange: exchange.name().to_string(),
                    price: level.price.to_f64().unwrap_or_default(),
                    amount: level.quantity.to_f64().unwrap_or_default(),
                });
                if let Some(ask_level) = self.books.get(&exchange).unwrap().asks.get(index) {
                    ask_heap.push(Reverse(BookLevelAndExchangeHelper(ask_level, exchange)));

                    ask_indices.insert_nocheck(exchange, index + 1);
                }
            }
        }
        let spread = if asks.first().is_some() && bids.first().is_some() {
            asks.first().as_ref().unwrap().price - bids.first().as_ref().unwrap().price
        } else {
            0.
        };
        Summary { spread, asks, bids }
    }
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::defines::grpc_scheme::Summary;
    use crate::defines::{Exchange, BOOK_LEVELS_USED};
    use crate::marketdata::{BookAggregator, BookLevel, Orderbook};
    use float_cmp::approx_eq;
    use halfbrown::HashMap;
    use rand::distributions::Distribution;
    use rand::distributions::Uniform;
    use rand::rngs::ThreadRng;
    use rand::Rng;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use smallvec::SmallVec;
    use std::collections::HashSet;

    const TEST_BOOKS_SIZE: usize = 10;

    /// This struct helps to generate random orderbooks
    /// such that prices are unique among all current books (this prevents sorting ambiguity)
    struct SummaryCorrectness<const N: usize = TEST_BOOKS_SIZE> {
        used_prices: HashSet<Decimal>,
        rng: ThreadRng,
        price_to_exchange: HashMap<Decimal, Exchange>,
    }

    impl<const N: usize> SummaryCorrectness<N> {
        pub fn new() -> Self {
            Self {
                used_prices: Default::default(),
                rng: Default::default(),
                price_to_exchange: Default::default(),
            }
        }
    }

    impl SummaryCorrectness {
        fn generate_random_unique_prices(
            &mut self,
            n: usize,
            min_price: Decimal,
            max_price: Decimal,
        ) -> Vec<Decimal> {
            let mut result = Vec::with_capacity(n);
            let uniform = Uniform::new(min_price, max_price);

            while result.len() < n {
                let price: Decimal = uniform.sample(&mut self.rng);
                if !self.used_prices.contains(&price) {
                    self.used_prices.insert(price);
                    result.push(price);
                }
            }
            assert_eq!(result.len(), n);
            result
        }

        fn prices_to_level_stubs(prices: Vec<Decimal>) -> SmallVec<[BookLevel; BOOK_LEVELS_USED]> {
            prices
                .into_iter()
                .map(|price| BookLevel {
                    price,
                    quantity: Default::default(),
                })
                .collect()
        }

        fn generate_random_book_levels(
            &mut self,
            n: usize,
            is_ask: bool,
        ) -> SmallVec<[BookLevel; BOOK_LEVELS_USED]> {
            let prices = if is_ask {
                let mut prices = self.generate_random_unique_prices(n, dec!(1), dec!(999));
                prices.sort();
                prices
            } else {
                let mut prices = self.generate_random_unique_prices(n, dec!(2000), dec!(2999));
                prices.sort_by(|a, b| b.cmp(a));
                prices
            };
            Self::prices_to_level_stubs(prices)
        }

        fn generate_random_book(&mut self, n: usize, exchange: Exchange) -> Orderbook {
            assert!(2 * n == self.price_to_exchange.len() || self.price_to_exchange.len() == 0);
            let bids = self.generate_random_book_levels(n, false);
            let asks = self.generate_random_book_levels(n, true);
            let n_prior = self.price_to_exchange.len();
            assert_eq!(bids.len() + asks.len(), 2 * n);
            for lvl in bids.iter().chain(asks.iter()) {
                assert!(self.price_to_exchange.insert(lvl.price, exchange).is_none())
            }
            assert_eq!(n_prior + 2 * n, self.price_to_exchange.len());

            Orderbook { bids, asks }
        }

        fn replace_book_with_new_random_one(
            &mut self,
            n: usize,
            book: Orderbook,
            exchange: Exchange,
        ) -> Orderbook {
            for bid in book.bids {
                self.used_prices.remove(&bid.price);
                self.price_to_exchange.remove(&bid.price);
            }
            for ask in book.asks {
                self.used_prices.remove(&ask.price);
                self.price_to_exchange.remove(&ask.price);
            }
            let res = self.generate_random_book(n, exchange);
            assert_eq!(self.used_prices.len(), 4 * n);
            assert_eq!(self.price_to_exchange.len(), 4 * n);
            res
        }
        fn check_spread_correctness(summary: &Summary) {
            let spread = summary.spread;
            let expected_spread = summary.asks.first().map(|x| x.price).unwrap_or_default()
                - summary.bids.first().map(|x| x.price).unwrap_or_default();
            assert!(approx_eq!(f64, spread, expected_spread, epsilon = 0.00001));
        }
        fn check_correctness_of_summary(
            &self,
            binance_book: &Orderbook,
            bitstamp_book: &Orderbook,
            summary: Summary,
        ) {
            Self::check_spread_correctness(&summary);
            let mut expected_bids: Vec<Decimal> = binance_book
                .bids
                .iter()
                .map(|x| x.price)
                .chain(bitstamp_book.bids.iter().map(|x| x.price))
                .collect();
            expected_bids.sort_by(|a, b| b.cmp(a));
            let mut expected_asks: Vec<Decimal> = binance_book
                .asks
                .iter()
                .map(|x| x.price)
                .chain(bitstamp_book.asks.iter().map(|x| x.price))
                .collect();
            expected_asks.sort();
            expected_asks.truncate(TEST_BOOKS_SIZE);
            expected_bids.truncate(TEST_BOOKS_SIZE);
            assert_eq!(
                summary.bids.len(),
                expected_bids.len(),
                "{binance_book:?} ||| {bitstamp_book:?} ||| {expected_bids:?}  ||| {summary:?}"
            );
            assert_eq!(summary.asks.len(), expected_asks.len());

            for (summary_bid, expected_bid) in summary.bids.iter().zip(expected_bids.iter()) {
                assert!(approx_eq!(
                    f64,
                    summary_bid.price,
                    expected_bid.to_f64().unwrap(),
                    epsilon = 0.00001
                ));
                assert_eq!(
                    summary_bid.exchange,
                    self.price_to_exchange
                        .get(expected_bid)
                        .unwrap()
                        .name()
                        .to_string()
                );
            }
            for (summary_ask, expected_ask) in summary.asks.iter().zip(expected_asks.iter()) {
                assert!(approx_eq!(
                    f64,
                    summary_ask.price,
                    expected_ask.to_f64().unwrap(),
                    epsilon = 0.00001
                ));
                assert_eq!(
                    summary_ask.exchange,
                    self.price_to_exchange
                        .get(expected_ask)
                        .unwrap()
                        .name()
                        .to_string()
                );
            }
        }
    }

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
            assert_eq!(
                a <= b,
                BookLevel {
                    price: a,
                    quantity: c,
                } <= BookLevel {
                    price: b,
                    quantity: d,
                }
            )
        }
    }

    #[test]
    fn make_summary_correctness() {
        let mut aggregator = BookAggregator::<TEST_BOOKS_SIZE>::new();
        let mut summary_correctness_helper = SummaryCorrectness::new();
        let mut rng = rand::thread_rng();
        //
        let mut bitstamp_book =
            summary_correctness_helper.generate_random_book(TEST_BOOKS_SIZE, Exchange::Bitstamp);

        let mut binance_book =
            summary_correctness_helper.generate_random_book(TEST_BOOKS_SIZE, Exchange::Binance);

        aggregator.add_new_book(binance_book.clone(), Exchange::Binance);
        aggregator.add_new_book(bitstamp_book.clone(), Exchange::Bitstamp);
        for _ in 0..2000 {
            let exchange_selector: bool = rng.gen();
            if exchange_selector {
                bitstamp_book = summary_correctness_helper.replace_book_with_new_random_one(
                    TEST_BOOKS_SIZE,
                    bitstamp_book,
                    Exchange::Bitstamp,
                );
                aggregator.add_new_book(bitstamp_book.clone(), Exchange::Bitstamp);
            } else {
                binance_book = summary_correctness_helper.replace_book_with_new_random_one(
                    TEST_BOOKS_SIZE,
                    binance_book,
                    Exchange::Binance,
                );
                aggregator.add_new_book(binance_book.clone(), Exchange::Binance);
            }
            let summary = aggregator.make_summary();
            summary_correctness_helper.check_correctness_of_summary(
                &binance_book,
                &bitstamp_book,
                summary,
            );
        }
    }
}
