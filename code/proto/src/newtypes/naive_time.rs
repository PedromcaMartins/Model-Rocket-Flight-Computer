use core::ops::Deref;

use chrono::NaiveTime;

use crate::{Serialize, Deserialize, Schema};
use crate::schema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NaiveTimeWrapper(NaiveTime);

impl Deref for NaiveTimeWrapper {
    type Target = NaiveTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NaiveTime> for NaiveTimeWrapper {
    fn from(value: NaiveTime) -> Self {
        Self(value)
    }
}

impl Schema for NaiveTimeWrapper {
    const SCHEMA: &'static schema::NamedType = &schema::NamedType {
        name: "NaiveTime",
        ty: &schema::DataModelType::Struct(&[
            &schema::NamedValue {
                name: "secs",
                ty: u32::SCHEMA,
            },
            &schema::NamedValue {
                name: "frac",
                ty: u32::SCHEMA,
            },
        ]),
    };
}

#[test]
fn fix_type_wrapping() {
    let time = NaiveTime::from_hms_micro_opt(12, 34, 56, 789012).unwrap();
    let wrapped = NaiveTimeWrapper::from(time);
    assert_eq!(time, *wrapped);
}
