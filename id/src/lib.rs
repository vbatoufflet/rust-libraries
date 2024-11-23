use std::{fmt, str};

use base32::Alphabet;
use uuid::Uuid;

use errors::prelude::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "sqlx")]
mod sqlx;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id(String, Uuid);

impl Id {
    #[must_use]
    pub fn new(prefix: &str) -> Self {
        Self(prefix.to_owned(), uuid::Uuid::now_v7())
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.1
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}_{}",
            self.0,
            base32::encode(Alphabet::Rfc4648Lower { padding: false }, self.1.as_bytes())
        )
    }
}

impl str::FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((prefix, id)) = s.split_once('_') else {
            return Err(Error::GroupCount);
        };

        let Some(uuid) = base32::decode(Alphabet::Rfc4648Lower { padding: false }, id) else {
            return Err(Error::Encoding);
        };

        Ok(Self(
            prefix.to_owned(),
            Uuid::from_slice(&uuid).map_err(|_| Error::Uuid)?,
        ))
    }
}

#[derive(Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("group count")]
    GroupCount,

    #[error("encoding")]
    Encoding,

    #[error("uuid")]
    Uuid,
}
