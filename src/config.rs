use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub accounts: Vec<Account>,
    pub log_level: String,
    pub log_directory: String
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub address: String,
    pub username: String,
    pub passowrd: String,
    pub port: u16,
    pub filters: Vec<Filter>,
}

#[derive(Deserialize, Debug)]
pub struct Filter {
    pub source: String,
    pub destination: String,
    pub selection: Selection,
    pub patterns: Vec<Pattern>,
}

#[derive(Deserialize, Debug)]
pub enum Pattern {
    From(String),
    To(String),
    Subject(String),
    Content(String),
}

#[derive(Deserialize, Debug)]
pub enum Selection {
    Unread(),
    All(),
    Latest(u32)
}

pub fn read_config(path: &str) -> Config {
    let conf = fs::read_to_string(path).unwrap();
    toml::from_str(&conf).unwrap()
}
