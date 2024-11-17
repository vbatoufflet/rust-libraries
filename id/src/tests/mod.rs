use std::str::FromStr;

use errors::prelude::*;
use uuid::Uuid;

use crate::Id;

#[tokio::test]
async fn new() -> Result<()> {
    let id = Id::new("prf");
    assert_eq!(id.prefix(), "prf");
    assert_eq!(id.to_string().len(), 30);

    Ok(())
}

#[tokio::test]
async fn parse() -> Result<()> {
    let result = Id::from_str("prf_agjttnzrr54jhngnwnpwhf42tu");
    assert!(result.is_ok());

    let id = result.unwrap();
    assert_eq!(id.prefix(), "prf");
    assert_eq!(
        id.uuid(),
        &Uuid::parse_str("019339b7-318f-7893-b4cd-b35f63979a9d").unwrap()
    );

    Ok(())
}
