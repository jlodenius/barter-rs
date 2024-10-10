use super::{book::l1::OkxOrderBookDataL1, trade::OkxMessage, Okx, OkxWebSocketParser};
use crate::{
    exchange::StreamSelector,
    subscription::book::{OrderBooksL1, OrderBooksL2},
    transformer::{book::MultiBookTransformer, stateless::StatelessTransformer},
};
use barter_integration::{
    model::instrument::Instrument, protocol::websocket::WsStream, ExchangeStream,
};

/// Level 1 OrderBook types (top of book) for perpetual futures
/// [`OrderBookUpdater`](crate::transformer::book::OrderBookUpdater) implementation.
pub mod l1;

/// Level 2 OrderBook types for perpetual futures
/// [`OrderBookUpdater`](crate::transformer::book::OrderBookUpdater) implementation.
pub mod l2;

impl StreamSelector<Instrument, OrderBooksL1> for Okx {
    type Stream = ExchangeStream<
        OkxWebSocketParser,
        WsStream,
        StatelessTransformer<Self, Instrument, OrderBooksL1, OkxMessage<OkxOrderBookDataL1>>,
    >;
}

impl StreamSelector<Instrument, OrderBooksL2> for Okx {
    type Stream = ExchangeStream<
        OkxWebSocketParser,
        WsStream,
        MultiBookTransformer<Self, Instrument, OrderBooksL2, l2::OkxFuturesBookUpdater>,
    >;
}
