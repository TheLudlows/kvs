use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use dashmap::DashMap;
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::options::{Options, ReadOptions};
use log::{info, log};
use tempfile::TempDir;
use crate::model::key::MyKey;
use crate::model::evn::*;
use crate::model::request::*;


#[derive(Debug, Clone)]
pub struct Kv {
    map: DashMap<MyKey, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ZSet {
    map: DashMap<String, SortValues>,
}

#[derive(Debug, Clone)]
struct SortValues {
    score_map: BTreeMap<u32, String>,
    value_map: HashMap<String, u32>
}

impl SortValues{
    pub fn new() -> Self {
        Self{
            score_map: Default::default(),
            value_map: Default::default()
        }
    }
}


impl ZSet {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn insert(&self, k: String, v: ScoreValue) {
        let mut e = self.map.entry(k.clone()).or_insert_with(|| SortValues::new());

        if let Some(v) = &e.score_map.insert(v.score, v.value.clone()) {
            &e.value_map.remove(v);
        }
        if let Some(v) = &e.value_map.insert(v.value, v.score) {
            &e.score_map.remove(v);
        }
    }

    pub fn range(&self, k:&String, range: ScoreRange) -> Vec<ScoreValue> {
        let mut ret = Vec::new();
        if let Some(vs) = self.map.get(k) {
            for v in vs.score_map.range(range.min_score..=range.max_score).into_iter() {
                ret.push(ScoreValue::new(*v.0, v.1.clone()))
            }
        }
        ret

    }

    pub fn remove(&self, k: &String, v: &String) {
        let mut e = self.map.entry(k.clone()).or_insert_with(|| SortValues::new());
        if let Some(v) = &e.value_map.remove(v) {
            &e.score_map.remove(v);
        }
    }
}

impl Kv {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn load_from_file(&self) {
        let mut pb = PathBuf::from(LEVEL_DB_PATH);
        if !pb.exists() {
           pb = PathBuf::from(LEVEL_DB_ONLINE_PATH);
        }
        let mut op = Options::new();
        op.create_if_missing = true;
        let mut database: Database<MyKey> = Database::open(pb.as_path(), op).unwrap();
        let mut it = database.iter(ReadOptions::new());
        while let Some((k, v)) = it.next() {
            self.map.insert(k, v);
        }
        info!("map size {}", self.map.len());
    }

    pub fn insert(&self, k: String, v: String) {
        self.map.insert(MyKey::from_string(k), v.into_bytes());
        //info!("map size {}", self.map.len());
    }

    pub fn del(&self, k: String) {
        self.map.remove(&MyKey::from_string(k));
    }

    pub fn get(&self, k: String) -> Option<String> {
        self.map.get(&MyKey::from_string(k))
            .map(|e| String::from(std::str::from_utf8(&e.value()[..]).unwrap()))
    }

    pub fn list(&self, keys: Vec<String>) -> Vec<InsrtRequest> {
        let mut vec = Vec::new();
        for k in keys {
            match self.get(k.clone()) {
                None => {}
                Some(v) => {
                    vec.push(InsrtRequest::new(k, v))
                }
            }
        }
        vec
    }

    pub fn batch_insert(&self, vs: Vec<InsrtRequest>) {
        for req in vs {
            self.insert(req.key, req.value);
        }
    }
}

#[test]
fn test_load_level_db(){
    let kv = Kv::new();
    kv.load_from_file();
}