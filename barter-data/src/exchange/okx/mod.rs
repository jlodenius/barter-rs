use self::{
    channel::OkxChannel, market::OkxMarket, subscription::OkxSubResponse, trade::OkxTrades,
};
use crate::{
    exchange::{Connector, ExchangeId, ExchangeSub, PingInterval, StreamSelector},
    instrument::InstrumentData,
    subscriber::{validator::WebSocketSubValidator, WebSocketSubscriber},
    subscription::trade::PublicTrades,
    transformer::stateless::StatelessTransformer,
};
use barter_integration::{
    error::SocketError,
    protocol::{
        websocket::{
            process_binary, process_close_frame, process_frame, process_ping, process_pong,
            WebSocket, WsError, WsMessage, WsStream,
        },
        StreamParser,
    },
    ExchangeStream,
};
use barter_macro::{DeExchange, SerExchange};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::debug;
use url::Url;

/// Defines the type that translates a Barter [`Subscription`](crate::subscription::Subscription)
/// into an exchange [`Connector`] specific channel used for generating [`Connector::requests`].
pub mod channel;

/// Defines the type that translates a Barter [`Subscription`](crate::subscription::Subscription)
/// into an exchange [`Connector`] specific market used for generating [`Connector::requests`].
pub mod market;

/// [`Subscription`](crate::subscription::Subscription) response type and response
/// [`Validator`](barter_integration::Validator) for [`Okx`].
pub mod subscription;

/// Public trade types for [`Okx`].
pub mod trade;

/// Futures types for [`Okx`].
pub mod futures;

/// OrderBook type for [`Okx`].
pub mod book;

/// [`Okx`] server base url.
///
/// See docs: <https://www.okx.com/docs-v5/en/#overview-api-resources-and-support>
pub const BASE_URL_OKX: &str = "wss://wsaws.okx.com:8443/ws/v5/public";

/// [`Okx`] server [`PingInterval`] duration.
///
/// See docs: <https://www.okx.com/docs-v5/en/#websocket-api-connect>
pub const PING_INTERVAL_OKX: Duration = Duration::from_secs(29);

/// [`Okx`] exchange.
///
/// See docs: <https://www.okx.com/docs-v5/en/#websocket-api>
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DeExchange, SerExchange,
)]
pub struct Okx;

impl Connector for Okx {
    const ID: ExchangeId = ExchangeId::Okx;
    type Channel = OkxChannel;
    type Market = OkxMarket;
    type Subscriber = WebSocketSubscriber;
    type SubValidator = WebSocketSubValidator;
    type SubResponse = OkxSubResponse;

    fn url() -> Result<Url, SocketError> {
        Url::parse(BASE_URL_OKX).map_err(SocketError::UrlParse)
    }

    fn ping_interval() -> Option<PingInterval> {
        Some(PingInterval {
            interval: tokio::time::interval(PING_INTERVAL_OKX),
            ping: || WsMessage::text("ping"),
        })
    }

    fn requests(exchange_subs: Vec<ExchangeSub<Self::Channel, Self::Market>>) -> Vec<WsMessage> {
        vec![WsMessage::Text(
            json!({
                "op": "subscribe",
                "args": &exchange_subs,
            })
            .to_string(),
        )]
    }
}

// Custom stream parser to handle OKX non-json plain text Pongs
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct OkxWebSocketParser;

impl StreamParser for OkxWebSocketParser {
    type Stream = WebSocket;
    type Message = WsMessage;
    type Error = WsError;

    fn parse<Output>(
        payload: Result<Self::Message, Self::Error>,
    ) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned,
    {
        match payload {
            Ok(ws_message) => match ws_message {
                // Custom okx handler for Text messages, since Pong's are sent as plain text and
                // are not valid json, and so can't be deserialized using serde_json.
                WsMessage::Text(payload) => match serde_json::from_str::<Output>(&payload) {
                    Ok(value) => Some(Ok(value)),
                    Err(error) => {
                        // Check if Pong only when initial de fails, this is the only custom logic
                        // for this parser
                        if payload == "pong" {
                            None
                        } else {
                            debug!(
                                ?error,
                                ?payload,
                                action = "returning Some(Err(err))",
                                "failed to deserialize WebSocket Message into domain specific Message"
                            );
                            Some(Err(SocketError::Deserialise { error, payload }))
                        }
                    }
                },
                // Reuse the parsers from barter-integration
                WsMessage::Binary(binary) => process_binary(binary),
                WsMessage::Ping(ping) => process_ping(ping),
                WsMessage::Pong(pong) => process_pong(pong),
                WsMessage::Close(close_frame) => process_close_frame(close_frame),
                WsMessage::Frame(frame) => process_frame(frame),
            },
            Err(ws_err) => Some(Err(SocketError::WebSocket(ws_err))),
        }
    }
}

impl<Instrument> StreamSelector<Instrument, PublicTrades> for Okx
where
    Instrument: InstrumentData,
{
    type Stream = ExchangeStream<
        OkxWebSocketParser,
        WsStream,
        StatelessTransformer<Self, Instrument::Id, PublicTrades, OkxTrades>,
    >;
}
