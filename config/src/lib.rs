pub mod prelude {
    pub use config_driver::ConfigError;

    pub use config_derive::Config;

    pub trait ConfigTrait: Sized {
        fn from_env(prefix: &str) -> Result<Self, ConfigError>;
    }
}

pub mod __internal {
    pub use config_driver::{Config, Environment};
}
