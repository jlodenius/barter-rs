use super::SubscriptionKind;
use barter_macro::{DeSubKind, SerSubKind};
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields
/// [`PublicAggregatedTrade`] [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DeSubKind, SerSubKind)]
pub struct PublicAggregatedTrades;

impl SubscriptionKind for PublicAggregatedTrades {
    type Event = PublicAggregatedTrade;
}

/// Normalised Barter [`PublicAggregatedTrade`] model. Works the same as
/// [`PublicTrade`](crate::subscription::trade::PublicTrade) but aggregates results for the same
/// timestamp into one [`MarketEvent<T>`](crate::event::MarketEvent) event.
///
/// Use case:
/// - Aggregating Candles
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct PublicAggregatedTrade {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub high: f64,
    pub low: f64,
}
