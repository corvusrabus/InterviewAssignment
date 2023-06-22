mod exchanges;
pub(crate) mod error;
pub(crate) mod json_parser;
pub(crate) mod grpc_scheme;

pub use exchanges::Exchange;

pub(crate) const BOOK_LEVELS_USED : usize =10;