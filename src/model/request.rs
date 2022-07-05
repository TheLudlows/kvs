
use serde::{self,Serialize, Deserialize};
#[derive(Debug,Clone,Serialize, Deserialize,Eq,PartialEq)]
pub struct InsrtRequest{
    pub key: String,
    pub value: String
}

impl InsrtRequest {
    pub fn new(key: String, value: String) -> Self {
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