mod json_messages;

use crate::defines::json_parser::{JSONError, JSONParser};
use crate::defines::Exchange;
use crate::feed::exchanges::binance::json_messages::{
    BinanceBookMessage, BookSubRequest, BookSubRequestResponse,
};
use crate::feed::ws_api_feed::OrderbookWsApi;
use crate::marketdata::Orderbook;
use log::error;
use url::Url;

const BINANCE_BOOK_DEPTH: usize = 10;

pub(in crate::feed) struct BinanceOrderbookWsApi {}

impl OrderbookWsApi for BinanceOrderbookWsApi {
    fn subscription_message(symbol: &str) -> String {
        JSONParser::to_string(&BookSubRequest::new(symbol)).unwrap()
    }

    fn verify_confirmation(symbol: &str, message: &str) -> bool {
        match JSONParser::from_str::<BookSubRequestResponse>(message) {
            Ok(x) => x.result == None,
            Err(e) => {
                error!(target : "BinanceFeed", "Unexpected json {message}; {e:?}");
                false
            }
        }
    }

    fn handle_message(msg: &str) -> Result<Orderbook, JSONError> {
        let binance_msg: BinanceBookMessage = JSONParser::from_str(msg)?;
        let book = Orderbook::new(binance_msg.data.bids, binance_msg.data.asks);
        Ok(book)
    }

    fn connection_endpoint() -> Url {
        Url::parse("wss://stream.binance.com:9443/stream").unwrap()
    }

    fn exchange() -> Exchange {
        Exchange::Binance
    }
}
