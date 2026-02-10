use std::str::FromStr;

use errors::prelude::*;
use uuid::Uuid;

use crate::Id;

#[tokio::test]
async fn new() -> Result<()> {
    let id = Id::new_unordered("prf");
    assert_eq!(id.prefix(), "prf");
    assert_eq!(id.to_string().len(), 29);

    Ok(())
}

#[tokio::test]
async fn parse() -> Result<()> {
    let result = Id::from_str("prf_03cwcoe5guuex91dd4hzrpzso");
    assert!(result.is_ok());

    let id = result.unwrap();
    assert_eq!(id.prefix(), "prf");
    assert_eq!(
        id.uuid(),
        &Uuid::parse_str("019361ea-0fa7-7e82-b254-c241d584f878").unwrap()
    );

    Ok(())
}

#[tokio::test]
async fn roundtrip_nil() -> Result<()> {
    let id = Id::from_slice("x", &[0u8; 16])?;
    let s = id.to_string();
    assert_eq!(s, "x_0000000000000000000000000");
    assert_eq!(Id::from_str(&s)?.uuid(), id.uuid());

    Ok(())
}

#[tokio::test]
async fn roundtrip_max() -> Result<()> {
    let id = Id::from_slice("prf", &[0xFFu8; 16])?;
    let s = id.to_string();
    assert_eq!(s, "prf_f5lxx1zz5pnorynqglhzmsp33");
    assert_eq!(Id::from_str(&s)?.uuid(), id.uuid());

    Ok(())
}

#[tokio::test]
async fn sort_order_preserved() -> Result<()> {
    let id1 = Id::new_ordered("prf");
    let id2 = Id::new_ordered("prf");
    assert!(id1.to_string() <= id2.to_string());

    Ok(())
}

#[tokio::test]
async fn missing_separator() {
    assert_eq!(
        Id::from_str("prf03cwcoe5guuex91dd4hzrpzso"),
        Err(crate::Error::GroupCount)
    );
}

#[tokio::test]
async fn wrong_length() {
    assert_eq!(
        Id::from_str("prf_03cwcoe5guuex91dd4hzrpzs"),
        Err(crate::Error::Encoding)
    );
    assert_eq!(
        Id::from_str("prf_03cwcoe5guuex91dd4hzrpzsoo"),
        Err(crate::Error::Encoding)
    );
}

#[tokio::test]
async fn invalid_character() {
    assert_eq!(
        Id::from_str("prf_03cwcoe5guuex91dd4hzrpzs!"),
        Err(crate::Error::Encoding)
    );
}

#[tokio::test]
async fn uppercase_rejected() {
    assert_eq!(
        Id::from_str("prf_03CWCOE5GUUEX91DD4HZRPZSO"),
        Err(crate::Error::Encoding)
    );
}
