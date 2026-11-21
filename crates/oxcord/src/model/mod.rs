//! Mappings of objects received from the API, re-exported from `oxcord_model`.

pub use oxcord_model::*;

pub mod prelude {
    pub use oxcord_model::prelude::*;

    pub use crate::error::{Error, Result};
    #[cfg(feature = "model")]
    pub use crate::ops::*;
}
