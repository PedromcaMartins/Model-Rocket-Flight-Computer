use derive_more::{Deref, From};

use crate::{Serialize, Deserialize, Schema};

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, From, Deref)]
pub struct Uid(u32);

#[cfg(feature = "embassy-time")]
mod impls {
    use core::sync::atomic::{AtomicU32, Ordering};

    use super::Uid;

    static UID_COUNTER: AtomicU32 = AtomicU32::new(0);

    impl Uid {
        #[inline]
        pub fn generate_id() -> Self {
            Self(UID_COUNTER.fetch_add(1, Ordering::Relaxed))
        }
    }
}
