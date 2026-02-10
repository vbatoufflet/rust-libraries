use serde::Deserialize;

use crate::{self as config, prelude::*};

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

#[derive(Config, Debug, Deserialize, PartialEq)]
struct ListConfig {
    #[config(default = DEFAULT_A)]
    a: String,

    #[config(default = DEFAULT_B_LIST, list = true)]
    b: Vec<String>,
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
async fn list_config_with_empty_list() {
    std::env::set_var("CONFIG_TEST_EMPTY_A", "def");
    std::env::set_var("CONFIG_TEST_EMPTY_B", "");

    let config = ListConfig::from_env("CONFIG_TEST_EMPTY").unwrap();

    assert_eq!(config.a, "def");
    assert_eq!(config.b, vec![""]);

    std::env::remove_var("CONFIG_TEST_EMPTY_A");
    std::env::remove_var("CONFIG_TEST_EMPTY_B");
}

#[tokio::test]
async fn list_config_with_single_item() {
    std::env::set_var("CONFIG_TEST_SINGLE_A", "def");
    std::env::set_var("CONFIG_TEST_SINGLE_B", "a");

    let config = ListConfig::from_env("CONFIG_TEST_SINGLE").unwrap();

    assert_eq!(config.a, "def");
    assert_eq!(config.b, vec!["a"]);

    std::env::remove_var("CONFIG_TEST_SINGLE_A");
    std::env::remove_var("CONFIG_TEST_SINGLE_B");
}

#[derive(Config, Debug, Deserialize, PartialEq)]
struct SkipPrefixConfig {
    #[config(default = DEFAULT_A)]
    a: String,

    #[config(default = DEFAULT_B, skip_prefix = true)]
    b: i32,
}

#[tokio::test]
async fn skip_prefix_field() {
    std::env::set_var("CONFIG_SKIP_PREFIX_A", "value");
    std::env::set_var("B", "456");

    let config = SkipPrefixConfig::from_env("CONFIG_SKIP_PREFIX").unwrap();

    assert_eq!(config.a, "value");
    assert_eq!(config.b, 456);

    std::env::remove_var("CONFIG_SKIP_PREFIX_A");
    std::env::remove_var("B");
}

#[derive(Config, Debug, Deserialize, PartialEq)]
struct SkipPrefixDefaultConfig {
    #[config(default = DEFAULT_A)]
    a: String,

    #[config(default = DEFAULT_B, skip_prefix = true)]
    c: i32,
}

#[tokio::test]
async fn skip_prefix_with_default() {
    let config = SkipPrefixDefaultConfig::from_env("CONFIG_SKIP_DEFAULT").unwrap();

    assert_eq!(config.a, DEFAULT_A);
    assert_eq!(config.c, DEFAULT_B);
}

#[derive(Config, Debug, Deserialize, PartialEq)]
struct SkipPrefixListConfig {
    #[config(default = DEFAULT_B_LIST, list = true, skip_prefix = true)]
    items: Vec<String>,
}

#[tokio::test]
async fn skip_prefix_list_field() {
    std::env::set_var("ITEMS", "x,y,z");

    let config = SkipPrefixListConfig::from_env("CONFIG_SKIP_LIST").unwrap();

    assert_eq!(config.items, vec!["x", "y", "z"]);

    std::env::remove_var("ITEMS");
}

#[derive(Config, Debug, Deserialize, PartialEq)]
struct SkipPrefixListDefaultConfig {
    #[config(default = DEFAULT_B_LIST, list = true, skip_prefix = true)]
    default_items: Vec<String>,
}

#[tokio::test]
async fn skip_prefix_list_with_default() {
    let config = SkipPrefixListDefaultConfig::from_env("CONFIG_SKIP_LIST_DEFAULT").unwrap();

    assert_eq!(config.default_items, DEFAULT_B_LIST);
}
