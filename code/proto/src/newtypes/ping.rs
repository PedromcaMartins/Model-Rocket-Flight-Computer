use derive_more::{Deref, From, Into};

use crate::{Serialize, Deserialize, Schema};

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, From, Into, Deref)]
pub struct PingRequest(u32);

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, From, Into, Deref)]
pub struct PingResponse(u32);

#[test]
fn ping_wrapping() {
    let value: u32 = 42;
    let request: PingRequest = value.into();
    let response = PingResponse::from(value);
    assert_eq!(value, *request);
    assert_eq!(value, *response);
}
