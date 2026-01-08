use derive_more::{Deref, From, Into};
use nmea::sentences::FixType;

use crate::{Serialize, Deserialize, Schema};
use crate::schema;

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, From, Into, Deref)]
pub struct FixTypeWrapper(FixType);

impl Schema for FixTypeWrapper {
    const SCHEMA: &'static schema::NamedType = &schema::NamedType {
        name: "FixType",
        ty: &schema::DataModelType::Enum(&[
            &schema::NamedVariant {
                name: "Invalid",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Gps",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "DGps",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Pps",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Rtk",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "FloatRtk",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Estimated",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Manual",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Simulation",
                ty: &schema::DataModelVariant::UnitVariant,
            },
        ]),
    };
}

#[test]
fn fix_type_wrapping() {
    let x = FixType::DGps;
    let y = FixTypeWrapper::from(x);
    assert_eq!(x, *y);
}
