use nmea::sentences::FixType;

use crate::{Serialize, Deserialize, Schema};
use crate::schema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FixTypeWrapper(FixType);

impl FixTypeWrapper {
    #[must_use]
    pub const fn into_inner(self) -> FixType {
        self.0
    }
}

impl From<FixType> for FixTypeWrapper {
    fn from(value: FixType) -> Self {
        Self(value)
    }
}

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
    let y = FixTypeWrapper::new(x.clone());
    assert_eq!(x, y.into_inner());
}
