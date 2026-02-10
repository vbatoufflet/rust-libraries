use std::fmt::{self, Write};

use uuid::Uuid;

use crate::Error;

const ALPHABET: [u8; 36] = *b"0123456789abcdefghijklmnopqrstuvwxyz";

const LENGTH: usize = 25;

pub fn read(s: &str) -> Result<Uuid, Error> {
    if s.len() != LENGTH {
        return Err(Error::Encoding);
    }

    let n = s.bytes().try_fold(0u128, |acc, b| {
        let val = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'z' => b - b'a' + 10,
            _ => return Err(Error::Encoding),
        };
        acc.checked_mul(36)
            .and_then(|n| n.checked_add(u128::from(val)))
            .ok_or(Error::Encoding)
    })?;

    Ok(Uuid::from_bytes(n.to_be_bytes()))
}

pub fn write(f: &mut fmt::Formatter<'_>, uuid: &Uuid) -> fmt::Result {
    let mut buf = [0u8; LENGTH];
    let mut n = u128::from_be_bytes(*uuid.as_bytes());
    for i in (0..LENGTH).rev() {
        buf[i] = ALPHABET[(n % 36) as usize];
        n /= 36;
    }
    for &b in &buf {
        f.write_char(b as char)?;
    }
    Ok(())
}
