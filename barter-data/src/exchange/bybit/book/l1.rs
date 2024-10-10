use super::BybitLevel;
use crate::{
    event::{MarketEvent, MarketIter},
    exchange::{
        bybit::{channel::BybitChannel, message::BybitMessage},
        subscription::ExchangeSub,
        ExchangeId,
    },
    subscription::book::{Level, OrderBookL1},
    Identifier,
};
use barter_integration::model::{Exchange, SubscriptionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BybitOrderBookL1Data {
    #[serde(alias = "s", deserialize_with = "de_ob_l1_subscription_id")]
    pub subscription_id: SubscriptionId,
    #[serde(alias = "b")]
    pub bids: Vec<BybitLevel>,
    #[serde(alias = "a")]
    pub asks: Vec<BybitLevel>,
    #[serde(alias = "u")]
    pub update_id: u64,
    #[serde(alias = "seq")]
    pub sequence: u64,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BybitOrderBookL1 {
    #[serde(
        alias = "ts",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc",
        default = "Utc::now"
    )]
    pub time: DateTime<Utc>,
    pub data: BybitOrderBookL1Data,
    #[serde(
        alias = "cts",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc",
        default = "Utc::now"
    )]
    pub created_time: DateTime<Utc>,
}

impl Identifier<Option<SubscriptionId>> for BybitMessage<BybitOrderBookL1> {
    fn id(&self) -> Option<SubscriptionId> {
        match self {
            BybitMessage::Trade(trade) => Some(trade.data.subscription_id.clone()),
            _ => None,
        }
    }
}

impl<InstrumentId> From<(ExchangeId, InstrumentId, BybitMessage<BybitOrderBookL1>)>
    for MarketIter<InstrumentId, OrderBookL1>
{
    fn from(
        (exchange_id, instrument, message): (
            ExchangeId,
            InstrumentId,
            BybitMessage<BybitOrderBookL1>,
        ),
    ) -> Self {
        match message {
            BybitMessage::Trade(book) => {
                let best_bid = book.data.bids.first().unwrap_or(&BybitLevel {
                    price: 0.0,
                    amount: 0.0,
                });
                let best_ask = book.data.asks.first().unwrap_or(&BybitLevel {
                    price: 0.0,
                    amount: 0.0,
                });

                Self(vec![Ok(MarketEvent {
                    exchange_time: book.time,
                    received_time: Utc::now(),
                    exchange: Exchange::from(exchange_id),
                    instrument,
                    kind: OrderBookL1 {
                        last_update_time: book.time,
                        best_bid: Level::from((best_bid.price, best_bid.amount)),
                        best_ask: Level::from((best_ask.price, best_ask.amount)),
                    },
                })])
            }
            BybitMessage::Response(_) => Self(vec![]),
        }
    }
}

/// Deserialize a [`BybitOrderBookL1`] "s" (eg/ "BTCUSDT") as the associated [`SubscriptionId`].
///
/// eg/ "orderbook.1.BTCUSDT"
pub fn de_ob_l1_subscription_id<'de, D>(deserializer: D) -> Result<SubscriptionId, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer)
        .map(|market| ExchangeSub::from((BybitChannel::ORDER_BOOK_L1, market)).id())
}
