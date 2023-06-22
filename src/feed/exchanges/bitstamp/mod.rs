mod json_messages;

use std::cmp::min;
use log::error;
use smallvec::{SmallVec, ToSmallVec};
use url::Url;
use crate::defines::{BOOK_LEVELS_USED, Exchange};
use crate::defines::json_parser::{JSONError, JSONParser};
use crate::feed::exchanges::bitstamp::json_messages::{BookMessage, BookSubRequest, BookSubRequestResponse};
use crate::feed::ws_api_feed::OrderbookWsApi;
use crate::marketdata::Orderbook;

pub(in crate::feed) struct BitstampOrderbookWsApi {}

impl OrderbookWsApi for BitstampOrderbookWsApi {
    fn subscription_message(symbol: &str) -> String {
        JSONParser::to_string(&BookSubRequest::new(symbol)).unwrap()
    }

    fn verify_confirmation(_symbol: &str, message: &str) -> bool {
        match JSONParser::from_str::<BookSubRequestResponse>(message) {
            Ok(x) => {
                &x.event == "bts:subscription_succeeded"
            }
            Err(e) => {
                error!(target : "BitstampFeed", "Unexpected json {message}; {e:?}");
                false
            }
        }
    }

    fn handle_message(msg: &str) -> Result<Orderbook, JSONError> {
        let bitstamp_msg: BookMessage = JSONParser::from_str(msg)?;
        let bid_size = min(BOOK_LEVELS_USED,bitstamp_msg.data.bids.len());
        let ask_size = min(BOOK_LEVELS_USED,bitstamp_msg.data.asks.len());
        let bids =bitstamp_msg.data.bids[..bid_size].to_smallvec();
        let asks = bitstamp_msg.data.asks[..ask_size].to_smallvec();
        let book = Orderbook::new( bids, asks );
        Ok(book)
    }

    fn connection_endpoint() -> Url {
        Url::parse("wss://ws.bitstamp.net/").unwrap()
    }

    fn exchange() -> Exchange {
        Exchange::Bitstamp
    }
}