#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Exchange {
    Binance,
    Bitstamp,
}

impl Exchange {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Exchange::Binance => "binance",
            Exchange::Bitstamp => "bitstamp",
        }
    }
}
