pub mod prelude {
    pub use anyhow::{anyhow, bail, ensure, Context as _, Result};
    pub use thiserror::Error;

    pub type AnyError = anyhow::Error;
}
