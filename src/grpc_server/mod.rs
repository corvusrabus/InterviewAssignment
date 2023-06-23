use crate::defines::grpc_scheme::orderbook_aggregator_server::OrderbookAggregator;
use crate::defines::grpc_scheme::{Empty, Summary};
use futures_util::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream, WatchStream};
use tonic::codegen::futures_core;
use tonic::Status;
#[derive(Debug, Default)]
pub(crate) struct GRPCServer {}

#[tonic::async_trait]
impl OrderbookAggregator for GRPCServer {
    type BookSummaryStream = BroadcastStreamDropWrapper<Result<Summary, Status>>;
    async fn book_summary(
        &self,
        request: tonic::Request<Empty>,
    ) -> std::result::Result<tonic::Response<Self::BookSummaryStream>, tonic::Status> {
        todo!()
    }
}

pub(crate) struct BroadcastStreamDropWrapper<T>(BroadcastStream<T>);

impl<T> Stream for BroadcastStreamDropWrapper<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}
