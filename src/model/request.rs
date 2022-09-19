use bytes::Bytes;
use serde::{self, Serialize, Deserialize};

#[derive(Debug,Clone,Serialize, Deserialize,Eq,PartialEq)]
pub struct InsrtRequest{
    pub key: Bytes,
    pub value: Bytes
}

#[derive(Debug,Clone,Serialize, Deserialize,Eq,PartialEq)]
pub struct Cluster{
    pub hosts:Vec<String>,
    pub index: usize
}

impl InsrtRequest {
    pub fn new(key: Bytes, value: Bytes) -> Self {
        InsrtRequest {
            key,
            value
        }
    }
}


#[derive(Debug,Clone,Serialize, Deserialize,Eq,PartialEq)]
pub struct ScoreValue{
    pub score: u32,
    pub value: String
}
impl ScoreValue {
    pub fn new(score: u32, value: String) -> Self {
        ScoreValue {
            score,
            value
        }
    }
}

#[derive(Debug,Clone,Serialize, Deserialize,Eq,PartialEq)]
pub struct ScoreRange{
    pub min_score: u32,
    pub max_score: u32
}

impl ScoreRange {
    pub fn new(min_score: u32, max_score: u32) -> Self {
        ScoreRange {
            min_score,
            max_score
        }
    }
}

use db_key::Key;


#[derive(Debug,Clone,Eq,PartialEq, Hash)]
pub struct MyKey(pub String);
impl Key for MyKey{
    fn from_u8(key: &[u8]) -> MyKey {
        MyKey(std::str::from_utf8(key).unwrap().to_string())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0.as_bytes())
    }
}

impl MyKey {
    pub fn from_string(v : String) -> Self {
        Self(v)
    }
}