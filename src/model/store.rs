use std::collections::{BTreeMap, HashMap};
use std::fs::create_dir;
use std::path::{PathBuf};

use dashmap::DashMap;
use fs_extra::dir::{copy, CopyOptions};
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::options::{Options, ReadOptions};
use log::{info};
use crate::model::{evn, http_req};
use crate::model::cluster::{CLUSTER_URL, IDX};
use crate::model::key::MyKey;
use crate::model::evn::*;
use crate::model::request::*;

#[derive(Debug, Clone)]
pub struct Kv {
    map_arr: Vec<DashMap<String, String>>,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct ZSet {
    map_arr: Vec<DashMap<String, SortValues>>,
    client: reqwest::Client,

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
            map_arr: vec,
            client: reqwest::Client::new(),
        }
    }

    pub async fn insert(&self, k: String, v: ScoreValue) {
        let cluster_idx = cluster_idx(&k);
        if cluster_idx == **IDX {
            self.do_insert(k, v);
        } else {
            match http_req::zadd(&self.client, &CLUSTER_URL[cluster_idx], k, v).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
    fn do_insert(&self, k: String, v: ScoreValue) {
        let mut map = &self.map_arr[shard_idx(&k)];
        let mut e = map.entry(k).or_insert_with(|| SortValues::new());

        if let Some(v) = &e.score_map.insert(v.score, v.value.clone()) {
            e.value_map.remove(v);
        }
        if let Some(v) = &e.value_map.insert(v.value, v.score) {
            e.score_map.remove(v);
        }
    }


    pub async fn range(&self, k: &String, range: ScoreRange) -> Vec<ScoreValue> {
        let mut ret = Vec::new();
        let cluster_idx = cluster_idx(k);
        if cluster_idx == **IDX {
            let map = &self.map_arr[shard_idx(&k)];
            if let Some(vs) = map.get(k) {
                for v in vs.score_map.range(range.min_score..=range.max_score).into_iter() {
                    ret.push(ScoreValue::new(*v.0, v.1.clone()))
                }
            }
        } else {
            match http_req::range(&self.client, &CLUSTER_URL[cluster_idx], k, range).await {
                Ok(v) => { ret.clone_from(&v) }
                Err(_) => {}
            }
        }
        ret
    }

    pub async fn remove(&self, k: &String, v: &String) {
        let cluster_idx = cluster_idx(k);
        if cluster_idx == **IDX {
            let map = &self.map_arr[shard_idx(&k)];
            if let Some(mut e) = map.get_mut(k) {
                if let Some(v) = &e.value_map.remove(v) {
                    e.score_map.remove(v);
                }
            }
        } else {
            match http_req::rmv(&self.client, &CLUSTER_URL[cluster_idx], k, v).await {
                Ok(_) => {}
                Err(_) => {}
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
            map_arr: vec,
            client: reqwest::Client::new(),
        }
    }

    pub fn load_from_file(&self) {
        let mut pb = PathBuf::from(BASE_PATH);

        pb.push(DATA_PATH);

        if !pb.exists() {
            //第一次加载
            create_dir(&pb).unwrap();
            let mut options = CopyOptions::new();
            options.skip_exist = true;

            copy(data_files[0], &pb, &options).unwrap();
            copy(data_files[1], &pb, &options).unwrap();
            copy(data_files[2], &pb, &options).unwrap();
            info!("copy dir to {:?}", pb);
        }
        let mut paths = vec![];
        for s in total_data_files {
            let mut data_path = PathBuf::from(&pb);
            data_path.push(s);
            paths.push(data_path)
        }

        if **IDX == 999 {
            return;
        }
        for pb in paths {
            let mut op = Options::new();
            let database: Database<MyKey> = Database::open(pb.as_path(), op).unwrap();
            let mut it = database.iter(ReadOptions::new());
            while let Some((k, v)) = it.next() {
                // 批量？
                self.insert_local(InsrtRequest::new(k.0, String::from_utf8(v).unwrap()));
            }
            info!("map size {}", self.map_arr.iter().map(|m| m.len()).sum::<usize>());
        }
    }

    #[inline]
    pub async fn insert(&self, req: InsrtRequest) {
        let cluster_idx = cluster_idx(&req.key);
        if cluster_idx == **IDX {
            self.map_arr[shard_idx(&req.key)].insert(req.key, req.value);
        } else {
            //info!("insert to {}, cur{}", cluster_idx, **IDX);
            match http_req::add(&self.client, &CLUSTER_URL[cluster_idx], req).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    #[inline]
    pub fn insert_local(&self, req: InsrtRequest) {
        let cluster_idx = cluster_idx(&req.key);
        if cluster_idx == **IDX {
            self.map_arr[shard_idx(&req.key)].insert(req.key, req.value);
        }
    }

    #[inline]
    pub async fn del(&self, k: &String) {
        let cluster_idx = cluster_idx(k);
        if cluster_idx == **IDX {
            self.map_arr[shard_idx(&k)].remove(k);
        } else {
            match http_req::del(&self.client, &CLUSTER_URL[cluster_idx], k).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    #[inline]
    pub async fn get(&self, k: &String) -> Option<String> {
        let cluster_idx = cluster_idx(k);
        return if cluster_idx == **IDX {
            (&self.map_arr[shard_idx(&k)]).get(k)
                //.map(|e| e.value().to_string())
                .map(|e| e.to_string())
        } else {
            //info!("get to {}, cur{}", cluster_idx, **IDX);
            match http_req::query(&self.client, &CLUSTER_URL[cluster_idx], k).await {
                Ok(v) => { Some(v) }
                Err(_) => {
                    // todo log
                    None
                }
            }
        };
    }
    #[inline]
    pub async fn list(&self, keys: Vec<String>) -> Vec<InsrtRequest> {
        let mut vec = Vec::new();
        for k in keys {
            match self.get(&k).await {
                None => {}
                Some(v) => {
                    vec.push(InsrtRequest::new(k, v))
                }
            }
        }
        vec
    }

    #[inline]
    pub async fn batch_insert(&self, vs: Vec<InsrtRequest>) {
        for req in vs {
            self.insert(req).await;
        }
    }
}

#[test]
fn test_load_level_db() {
    let kv = Kv::new();
    kv.load_from_file();
}