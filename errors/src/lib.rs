pub mod prelude {
    pub use anyhow::{anyhow, bail, ensure, Context as _, Result};
    pub use thiserror::{self, Error};

    pub type AnyError = anyhow::Error;
}
