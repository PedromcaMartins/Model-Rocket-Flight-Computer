use chrono::NaiveTime;
use derive_more::{Deref, From, Into};

use crate::{Serialize, Deserialize, Schema};
use crate::schema;

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, From, Into, Deref)]
pub struct NaiveTimeWrapper(NaiveTime);

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
