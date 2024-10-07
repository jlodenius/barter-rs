use crate::{
    event::{MarketEvent, MarketIter},
    exchange::{
        okx::{book::l1::OkxOrderBookDataL1, trade::OkxMessage},
        ExchangeId,
    },
    subscription::book::OrderBookL1,
};
use barter_integration::model::Exchange;
use chrono::Utc;

impl<InstrumentId: Clone> From<(ExchangeId, InstrumentId, OkxMessage<OkxOrderBookDataL1>)>
    for MarketIter<InstrumentId, OrderBookL1>
{
    fn from(
        (exchange_id, instrument, book): (ExchangeId, InstrumentId, OkxMessage<OkxOrderBookDataL1>),
    ) -> Self {
        let events = book
            .data
            .into_iter()
            .map(|data| {
                Ok(MarketEvent {
                    exchange_time: data.time,
                    received_time: Utc::now(),
                    exchange: Exchange::from(exchange_id),
                    instrument: instrument.clone(),
                    kind: OrderBookL1::from(data),
                })
            })
            .collect::<Vec<_>>();
        Self(events)
    }
}
