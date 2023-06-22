#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Exchange {
    Binance,
    Bitstamp,
}

impl Exchange {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Exchange::Binance => { "binance" }
            Exchange::Bitstamp => { "bitstamp" }
        }
    }
}

// impl ToString for Exchange {
//     fn to_string(&self) -> String {
//         match self {
//             Exchange::Binance => { "binance" }
//             Exchange::Bitstamp => { "bitstamp" }
//         }.to_string()
//     }
// }