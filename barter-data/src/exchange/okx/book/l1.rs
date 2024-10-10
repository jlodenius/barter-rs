use super::OkxLevel;
use crate::subscription::book::OrderBookL1;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct OkxOrderBookDataL1 {
    #[serde(
        alias = "ts",
        deserialize_with = "barter_integration::de::de_str_u64_epoch_ms_as_datetime_utc"
    )]
    pub time: DateTime<Utc>,
    pub asks: Vec<OkxLevel>,
    pub bids: Vec<OkxLevel>,
    #[serde(rename = "seqId")]
    pub seq_id: i64,
}

impl From<OkxOrderBookDataL1> for OrderBookL1 {
    fn from(data: OkxOrderBookDataL1) -> Self {
        Self {
            last_update_time: Utc::now(),
            best_bid: data.bids[0].into(),
            best_ask: data.asks[0].into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barter_integration::model::SubscriptionId;

    mod de {
        use crate::exchange::okx::trade::OkxMessage;

        use super::*;

        #[test]
        fn test_okx_order_book_l1() {
            let input = r#"
            {
              "arg": {
                "channel": "bbo-tbt",
                "instId": "BCH-USDT-SWAP"
              },
              "data": [
                {
                  "asks": [
                    [
                      "111.06","55154","0","2"
                    ]
                  ],
                  "bids": [
                    [
                      "111.05","57745","0","2"
                    ]
                  ],
                  "ts": "1670324386802",
                  "seqId": 363996337
                }
              ]
            }
            "#;

            assert_eq!(
                serde_json::from_str::<OkxMessage<OkxOrderBookDataL1>>(input).unwrap(),
                OkxMessage {
                    subscription_id: SubscriptionId::from("bbo-tbt|BCH-USDT-SWAP"),
                    data: vec![OkxOrderBookDataL1 {
                        time: DateTime::<Utc>::from_timestamp_millis(1670324386802).unwrap(),
                        asks: vec![OkxLevel {
                            price: 111.06,
                            amount: 55154.0,
                        }],
                        bids: vec![OkxLevel {
                            price: 111.05,
                            amount: 57745.0,
                        }],
                        seq_id: 363996337,
                    }]
                }
            )
        }
    }
}
