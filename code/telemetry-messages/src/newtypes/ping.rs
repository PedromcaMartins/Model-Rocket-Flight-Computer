use core::ops::Deref;

use crate::{Serialize, Deserialize, Schema, Format};

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, Format, PartialEq, Eq)]
pub struct PingRequest(u32);

impl From<u32> for PingRequest {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Deref for PingRequest {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, Format, PartialEq, Eq)]
pub struct PingResponse(u32);

impl From<u32> for PingResponse {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Deref for PingResponse {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test]
fn ping_wrapping() {
    let value: u32 = 42;
    let request = PingRequest::from(value);
    let response = PingResponse::from(value);
    assert_eq!(value, *request);
    assert_eq!(value, *response);
}
