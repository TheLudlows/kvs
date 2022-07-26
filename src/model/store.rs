use std::collections::{BTreeMap, HashMap};
use std::path::{PathBuf};
use bytes::Bytes;

use dashmap::{DashMap};
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::options::{Options, ReadOptions};
use log::{info};
use crate::model::key::MyKey;
use crate::model::evn::*;
use crate::model::request::*;


#[derive(Debug, Clone)]
pub struct Kv {
    map_arr: Vec<DashMap<Bytes, Bytes>>,
}

#[derive(Debug, Clone)]
pub struct ZSet {
    map_arr: Vec<DashMap<String, SortValues>>,
}

#[derive(Debug, Clone)]
struct SortValues {
    score_map: BTreeMap<u32, String>,
    value_map: HashMap<String, u32>,
}

impl SortValues {
    pub fn new() -> Self {
        Self {
            score_map: Default::default(),
            value_map: Default::default(),
        }
    }
}


impl ZSet {
    pub fn new() -> Self {
        let mut vec = Vec::new();
        for _ in 0..SHARD_NUM {
            vec.push(DashMap::with_capacity(DEFAULT_SIZE));
        }
        Self {
            map_arr: vec
        }
    }

    pub fn insert(&self, k: String, v: ScoreValue) {
        let map = &self.map_arr[shard_idx(&k.as_bytes())];
       /* map.alter(k, |value| {
            let mut score = match value {
                Some(v) => {
                    v
                }
                None => {
                    SortValues::new()
                }
            };
            score.score_map.insert(v.score, v.value.clone());
            score.value_map.insert(v.value, v.score);
            Some(score)
        });*/


        let mut e = map.entry(k).or_insert_with(|| SortValues::new());

        if let Some(v) = &e.score_map.insert(v.score, v.value.clone()) {
            e.value_map.remove(v);
        }
        if let Some(v) = &e.value_map.insert(v.value, v.score) {
            e.score_map.remove(v);
        }
    }

    pub fn range(&self, k: &String, range: ScoreRange) -> Vec<ScoreValue> {
        let map = &self.map_arr[shard_idx(&k.as_bytes())];

        let mut ret = Vec::new();
        if let Some(vs) = map.get(k) {
            for v in vs.score_map.range(range.min_score..=range.max_score).into_iter() {
                ret.push(ScoreValue::new(*v.0, v.1.clone()))
            }
        }
        ret
    }

    pub fn remove(&self, k: &String, v: &String) {
        let map = &self.map_arr[shard_idx(&k.as_bytes())];
        /*map.alter(k.to_string(), |value| {
            match value {
                None => {
                    None
                }
                Some(mut score_values) => {
                    if let Some(score) = score_values.value_map.remove(v) {
                        score_values.score_map.remove(&score);
                    }
                    Some(score_values)
                }
            }
        });*/
        if let Some(mut e) = map.get_mut(k) {
            if let Some(v) = &e.value_map.remove(v) {
                e.score_map.remove(v);
            }
        }
    }
}

impl Kv {
    pub fn new() -> Kv {
        let mut vec = Vec::new();
        for _ in 0..SHARD_NUM {
            vec.push(DashMap::with_capacity(DEFAULT_KV_SIZE));
        }
        Self {
            map_arr: vec
        }
    }

    pub fn load_from_file(&self) {
        let mut pb = PathBuf::from(LEVEL_DB_PATH);
        if !pb.exists() {
            pb = PathBuf::from(LEVEL_DB_ONLINE_PATH);
        }
        let mut op = Options::new();
        op.create_if_missing = true;
        let database: Database<MyKey> = Database::open(pb.as_path(), op).unwrap();
        let mut it = database.iter(ReadOptions::new());
        while let Some((k, v)) = it.next() {
            self.insert(Bytes::from(k.0), Bytes::from(v));
        }
        info!("map size {}", self.map_arr.iter().map(|m| m.len()).sum::<usize>());
    }

    #[inline]
    pub fn insert(&self, k: Bytes, v: Bytes) {
        self.map_arr[shard_idx(&k)].insert(k, v);
        //info!("map size {}", self.map.len());
    }

    #[inline]
    pub fn del(&self, k: &Bytes) {
        self.map_arr[shard_idx(&k)].remove(k);
    }

    #[inline]
    pub fn get(&self, k: &Bytes) -> Option<Bytes> {
        (&self.map_arr[shard_idx(&k)]).get(k)
            //.map(|e| e.value().to_string())
            .map(|e| e.clone())
    }
    #[inline]
    pub fn list(&self, keys: Vec<Bytes>) -> Vec<InsrtRequest> {
        let mut vec = Vec::new();
        for k in keys {
            match self.get(&k) {
                None => {}
                Some(v) => {
                    vec.push(InsrtRequest::new(k, v))
                }
            }
        }
        vec
    }

    #[inline]
    pub fn batch_insert(&self, vs: Vec<InsrtRequest>) {
        for req in vs {
            self.insert(req.key, req.value);
        }
    }
}

#[test]
fn test_load_level_db() {
    let kv = Kv::new();
    kv.load_from_file();
}