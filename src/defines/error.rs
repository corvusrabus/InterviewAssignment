use crate::defines::json_parser::JSONError;
use tokio_tungstenite::tungstenite::Error as TTError;

pub type WebsocketResult<T> = Result<T, WebsocketError>;
#[derive(Debug)]
pub enum WebsocketError {
    TungsteniteError(TTError),
    Timeout,
    UnexpectedClosure,
    NoConfirmationReceived,
    JsonError(JSONError),
}

impl From<TTError> for WebsocketError {
    fn from(value: TTError) -> Self {
        Self::TungsteniteError(value)
    }
}
impl From<JSONError> for WebsocketError {
    fn from(value: JSONError) -> Self {
        Self::JsonError(value)
    }
}
