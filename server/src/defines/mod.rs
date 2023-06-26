pub(crate) mod book_callback;
pub(crate) mod error;
mod exchanges;
pub(crate) mod grpc_scheme;
pub(crate) mod json_parser;

pub use exchanges::Exchange;

pub(crate) const BOOK_LEVELS_USED: usize = 10;
