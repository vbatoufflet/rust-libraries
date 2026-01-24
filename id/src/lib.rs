#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "sqlx")]
mod sqlx;
#[cfg(test)]
mod tests;

use std::{borrow::Cow, fmt, str};

use base32::Alphabet;
use uuid::Uuid;

use errors::prelude::*;

#[derive(Clone, Eq, PartialEq)]
pub struct Id(Cow<'static, str>, Uuid);

impl Id {
    #[must_use]
    pub fn new_ordered(prefix: &'static str) -> Self {
        Self(Cow::Borrowed(prefix), Uuid::now_v7())
    }

    #[must_use]
    pub fn new_unordered(prefix: &'static str) -> Self {
        Self(Cow::Borrowed(prefix), Uuid::new_v4())
    }

    pub fn from_slice(prefix: &str, b: &[u8]) -> Result<Self, Error> {
        let uuid = Uuid::from_slice(b).map_err(|_| Error::Uuid)?;
        Ok(Self(Cow::Owned(prefix.to_owned()), uuid))
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.1
    }

    fn fmt(&self) -> String {
        format!(
            "{}_{}",
            self.0,
            base32::encode(Alphabet::Crockford, self.1.as_bytes()).to_lowercase()
        )
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.fmt())
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fmt())
    }
}

impl str::FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((prefix, id)) = s.split_once('_') else {
            return Err(Error::GroupCount);
        };

        let Some(uuid) = base32::decode(Alphabet::Crockford, &id.to_uppercase()) else {
            return Err(Error::Encoding);
        };

        Ok(Self(
            Cow::Owned(prefix.to_owned()),
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
