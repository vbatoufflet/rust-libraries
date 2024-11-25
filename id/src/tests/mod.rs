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
    let result = Id::from_str("prf_069p3tgfmxz85cjmr90xb17rf0");
    assert!(result.is_ok());

    let id = result.unwrap();
    assert_eq!(id.prefix(), "prf");
    assert_eq!(
        id.uuid(),
        &Uuid::parse_str("019361ea-0fa7-7e82-b254-c241d584f878").unwrap()
    );

    Ok(())
}
