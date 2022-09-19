use std::collections::{BTreeMap, HashMap};
use std::path::{PathBuf};
use std::sync::atomic::Ordering;
use bytes::Bytes;

use dashmap::DashMap;
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::options::{Options, ReadOptions};
use log::{error, info};
use crate::model::{http_req};
use crate::model::cluster::{CLUSTER_URL, IDX, LOADED};
use crate::model::evn::*;
use crate::model::request::*;

#[derive(Debug, Clone)]
pub struct Store {
    map_arr: Vec<DashMap<Bytes, Bytes>>,
    zset_arr: Vec<DashMap<String, SortValues>>,
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


impl Store {
    pub fn new() -> Store {
        let mut vec = Vec::new();
        for _ in 0..SHARD_NUM {
            vec.push(DashMap::with_capacity(DEFAULT_KV_SIZE));
        }
        let mut set_vec = Vec::new();
        for _ in 0..SHARD_NUM {
            vec.push(DashMap::with_capacity(DEFAULT_SIZE));
        }
        Self {
            map_arr: vec,
            zset_arr: set_vec,
            client: reqwest::Client::new(),
        }
    }

    pub fn load_from_file(&self) -> String {
        if LOADED.load(Ordering::Acquire) {
            return String::from("ok");
        }
        if **IDX == 999 {
            info!("no cluster info");
            return String::from("no cluster info");
        }

        let mut paths = vec![];
        for ps in data_files {
            paths.push(PathBuf::from(ps));
        }

        for pb in paths {
            info!("load data from {:?}", pb);
            let op = Options::new();
            //op.create_if_missing = true;
            let result: Result<Database<MyKey>, _> = Database::open(pb.as_path(), op);
            if !result.is_ok() {
                info!("open {:?}, failed {:?}", pb, result.err());
                return format!("open failed");
            }
            let database = result.unwrap();
            let mut it = database.iter(ReadOptions::new());
            while let Some((k, v)) = it.next() {
                // 批量？
                let ins = InsrtRequest::new(Bytes::from(k.0), Bytes::from(v));
                //info!("{:?}",ins );
                self.insert_local(ins);
            }
            info!("map size {}", self.map_arr.iter().map(|m| m.len()).sum::<usize>());
        }

        LOADED.store(true, Ordering::Release);
        return String::from("ok");
    }
}

impl Store {
    pub async fn zset_insert(&self, k: String, v: ScoreValue) {
        let cluster_idx = cluster_idx(&k.as_bytes());
        if cluster_idx == **IDX {
            self.do_insert(k, v);
        } else {
            match http_req::zadd(&self.client, &CLUSTER_URL[cluster_idx], k, v).await {
                Ok(_) => {}
                Err(_) => {
                    error!("insert err")
                }
            }
        }
        //info!("map size {}", self.map_arr.iter().map(|m| m.len()).sum::<usize>());
    }
    fn do_insert(&self, k: String, v: ScoreValue) {
        let map = &self.zset_arr[shard_idx(&k.as_bytes())];
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
        let cluster_idx = cluster_idx(k.as_bytes());
        if cluster_idx == **IDX {
            let map = &self.zset_arr[shard_idx(k.as_bytes())];
            if let Some(vs) = map.get(k) {
                for v in vs.score_map.range(range.min_score..=range.max_score).into_iter() {
                    ret.push(ScoreValue::new(*v.0, v.1.clone()))
                }
            }
        } else {
            match http_req::range(&self.client, &CLUSTER_URL[cluster_idx], k, range).await {
                Ok(v) => { ret.clone_from(&v) }
                Err(_) => {
                    error!("range err")

                }
            }
        }
        ret
    }

    pub async fn remove(&self, k: &String, v: &String) {
        let cluster_idx = cluster_idx(k.as_bytes());
        if cluster_idx == **IDX {
            let map = &self.zset_arr[shard_idx(k.as_bytes())];
            if let Some(mut e) = map.get_mut(k) {
                if let Some(v) = &e.value_map.remove(v) {
                    e.score_map.remove(v);
                }
            }
        } else {
            match http_req::rmv(&self.client, &CLUSTER_URL[cluster_idx], k, v).await {
                Ok(_) => {}
                Err(_) => {
                    error!("rmv err")

                }
            }
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
                Err(_) => {
                    error!("insert kv err")
                }
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
        let cluster_idx = cluster_idx(k.as_bytes());
        if cluster_idx == **IDX {
            self.map_arr[shard_idx(k.as_bytes())].remove(k.as_bytes());
        } else {
            match http_req::del(&self.client, &CLUSTER_URL[cluster_idx], k).await {
                Ok(_) => {}
                Err(_) => {
                    error!("del err")

                }
            }
        }
    }

    #[inline]
    pub async fn get(&self, k: &Bytes) -> Option<String> {
        let cluster_idx = cluster_idx(&k);
        return if cluster_idx == **IDX {
           self.local_get(k)
        } else {
            //info!("get to {}, cur{}", cluster_idx, **IDX);
            match http_req::query(&self.client, &CLUSTER_URL[cluster_idx], &String::from_utf8(k.to_vec()).unwrap()).await {
                Ok(v) => {
                    match v {
                        None => None,
                        Some(val) => { Some(val) }
                    }
                }

                _ => {
                    // todo log
                    None
                }
            }
        };
    }

    pub fn local_get(&self, k :&Bytes) -> Option<String> {
        (&self.map_arr[shard_idx(&k)]).get(&k)
            //.map(|e| e.value().to_string())
            .map(|e| e.to_string())
    }
    #[inline]
    pub async fn list(&self, keys: Vec<Bytes>) -> Vec<InsrtRequest> {
        let mut ret = Vec::new();
        let mut vec_group = vec![];
        for _ in 0..CLUSTER_NUM {
            vec_group.push(vec![]);
        }

        for k in keys {
            let cluster_idx = cluster_idx(&k);
            vec_group[cluster_idx].push(k);
        }

        for i in 0..vec_group.len() {
            if vec_group[i].is_empty() {
                continue;
            }
            if i == **IDX {
                for k in vec_group[i].iter() {
                    match self.local_get(k) {
                        None => {}
                        Some(v) => {
                            ret.push(InsrtRequest::new(Bytes::from(k), Bytes::from(v)));
                        }
                    }
                }
            } else {
                match http_req::list(&self.client, &CLUSTER_URL[i], &vec_group[i]).await {
                    Ok(mut res) => {
                        while let Some(v) = res.pop() {
                            ret.push(v)
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        ret
    }

    #[inline]
    pub async fn batch_insert(&self, reqs: Vec<InsrtRequest>) {
        let mut vec_group = vec![];
        for _ in 0..CLUSTER_NUM {
            vec_group.push(vec![]);
        }
        for req in reqs {
            let cluster_idx = cluster_idx(&req.key);
            vec_group[cluster_idx].push(req);
        }

        for i in 0..vec_group.len() {
            if vec_group[i].is_empty() {
                continue;
            }
            if i == **IDX {
                for req in vec_group[i].clone() {
                    self.insert_local(req);
                }
            } else {
                match http_req::batch(&self.client, &CLUSTER_URL[i], vec_group[i].clone()).await {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    }
}

#[test]
fn test_load_level_db() {
    let kv = Store::new();
    kv.load_from_file();
}