#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, trace, warn};


#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn};
