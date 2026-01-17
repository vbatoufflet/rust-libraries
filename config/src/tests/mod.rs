use serde::Deserialize;

use crate as config;
use crate::prelude::*;

const DEFAULT_A: &str = "abc";

const DEFAULT_B: i32 = 123;

const DEFAULT_B_LIST: &[&str] = &["a", "b", "c"];

#[derive(Config, Debug, Deserialize, PartialEq)]
struct BasicConfig {
    #[config(default = DEFAULT_A)]
    a: String,

    #[config(default = DEFAULT_B)]
    b: i32,
}

#[derive(Config, Debug, Deserialize, PartialEq)]
struct ListConfig {
    #[config(default = DEFAULT_A)]
    a: String,

    #[config(default = DEFAULT_B_LIST, list = true)]
    b: Vec<String>,
}

#[tokio::test]
async fn basic_config_defaults() {
    let config = BasicConfig::from_env("CONFIG_TEST_DEFAULTS").unwrap();

    assert_eq!(config.a, DEFAULT_A);
    assert_eq!(config.b, DEFAULT_B);
}

#[tokio::test]
async fn basic_config_overrides() {
    std::env::set_var("CONFIG_TEST_FROM_ENV_A", "def");
    std::env::set_var("CONFIG_TEST_FROM_ENV_B", "456");

    let config = BasicConfig::from_env("CONFIG_TEST_FROM_ENV").unwrap();

    assert_eq!(config.a, "def");
    assert_eq!(config.b, 456);

    std::env::remove_var("CONFIG_TEST_FROM_ENV_A");
    std::env::remove_var("CONFIG_TEST_FROM_ENV_B");
}

#[tokio::test]
async fn list_config_defaults() {
    let config = ListConfig::from_env("CONFIG_TEST_LIST_DEFAULTS").unwrap();

    assert_eq!(config.a, DEFAULT_A);
    assert_eq!(config.b, DEFAULT_B_LIST);
}

#[tokio::test]
async fn list_config_overrides() {
    std::env::set_var("CONFIG_TEST_LIST_A", "def");
    std::env::set_var("CONFIG_TEST_LIST_B", "d,e,f");

    let config = ListConfig::from_env("CONFIG_TEST_LIST").unwrap();

    assert_eq!(config.a, "def");
    assert_eq!(config.b, vec!["d", "e", "f"]);

    std::env::remove_var("CONFIG_TEST_LIST_A");
    std::env::remove_var("CONFIG_TEST_LIST_B");
}

#[tokio::test]
async fn list_config_empty_list() {
    std::env::set_var("CONFIG_TEST_EMPTY_A", "def");
    std::env::set_var("CONFIG_TEST_EMPTY_B", "");

    let config = ListConfig::from_env("CONFIG_TEST_EMPTY").unwrap();

    assert_eq!(config.a, "def");
    assert_eq!(config.b, vec![""]);

    std::env::remove_var("CONFIG_TEST_EMPTY_A");
    std::env::remove_var("CONFIG_TEST_EMPTY_B");
}

#[tokio::test]
async fn list_config_single_item() {
    std::env::set_var("CONFIG_TEST_SINGLE_A", "def");
    std::env::set_var("CONFIG_TEST_SINGLE_B", "a");

    let config = ListConfig::from_env("CONFIG_TEST_SINGLE").unwrap();

    assert_eq!(config.a, "def");
    assert_eq!(config.b, vec!["a"]);

    std::env::remove_var("CONFIG_TEST_SINGLE_A");
    std::env::remove_var("CONFIG_TEST_SINGLE_B");
}
