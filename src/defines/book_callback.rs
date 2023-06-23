use crate::defines::Exchange;
use crate::marketdata::Orderbook;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub(crate) trait BookCallback: Send + Sync + 'static {
    async fn accept_book(&self, book: Orderbook, exchange: Exchange);
}
#[async_trait]
impl<T: BookCallback> BookCallback for Arc<T> {
    async fn accept_book(&self, book: Orderbook, exchange: Exchange) {
        BookCallback::accept_book(self.as_ref(), book, exchange).await
    }
}
