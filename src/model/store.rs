use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{PathBuf};
use std::sync::atomic::Ordering;

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
    map_arr: Vec<DashMap<String, String>>,
    zset_arr: Vec<DashMap<String, SortValues>>,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
struct SortValues {
    score_map: BTreeMap<u32, HashSet<String>>,
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
            set_vec.push(DashMap::with_capacity(DEFAULT_SIZE));
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
           /* while let Some((k, v)) = it.next() {
                if cluster_idx(&k.0) == **IDX {
                    // 批量？
                    //let ins = InsrtRequest::new(k.0, String::from_utf8(v).unwrap());
                    //info!("{:?}",ins );
                    //self.insert_local(ins);
                }
            }
            info!("map size {}", self.map_arr.iter().map(|m| m.len()).sum::<usize>());*/
        }

        LOADED.store(true, Ordering::Release);
        return String::from("ok");
    }
}

impl Store {
    pub async fn zset_insert(&self, k: String, v: ScoreValue) {
        let cur_id = **IDX;
        self.do_insert(k.clone(), v.clone());

        let mut vec = vec![];

        for i in 0..CLUSTER_NUM {
            if i == cur_id {
                continue;
            }
            vec.push(http_req::zadd(&self.client, &CLUSTER_URL[i], k.clone(), v.clone()));
        }
        futures::future::join_all(vec).await;
    }
    pub fn do_insert(&self, k: String, v: ScoreValue) {
        let map = &self.zset_arr[shard_idx(&k)];
        let mut e = map.entry(k).or_insert_with(|| SortValues::new());

        if let Some(old_score) = &e.value_map.insert(v.value.clone(), v.score) {
            if let Some(v_set) = e.score_map.get_mut(old_score) {
                v_set.remove(&v.value);
            }
        }

        let v_set = e.score_map.entry(v.score).or_insert(HashSet::new());

        v_set.insert(v.value);
    }

    pub fn range_Local(&self, k: &String, range: ScoreRange) -> Vec<ScoreValue> {
        let mut ret = Vec::new();
        let map = &self.zset_arr[shard_idx(k)];
        if let Some(vs) = map.get(k) {
            for v in vs.score_map.range(range.min_score..=range.max_score).into_iter() {
                for val in v.1.iter() {
                    ret.push(ScoreValue::new(*v.0, val.clone()));
                }
            }
        }
        ret
    }

    pub fn range(&self, k: &String, range: ScoreRange) -> Vec<ScoreValue> {
        self.range_Local(k, range)
    }

    pub async fn remove(&self, k: &String, v: &String) {
        let map = &self.zset_arr[shard_idx(&k)];
        if let Some(mut e) = map.get_mut(k) {
            if let Some(old_score) = &e.value_map.remove(v) {
                if let Some(v_set) = e.score_map.get_mut(old_score) {
                    v_set.remove(v);
                }
            }
        }
       /* let cur_id = **IDX;

        let mut vec = vec![];
        for i in 0..CLUSTER_NUM {
            if i == cur_id {
                continue;
            }
            vec.push(http_req::rmv(&self.client, &CLUSTER_URL[i], k, v));
        }
        futures::future::join_all(vec).await;*/
    }

    pub fn remove2(&self, k: &String, v: &String) {
        let map = &self.zset_arr[shard_idx(&k)];
        if let Some(mut e) = map.get_mut(k) {
            if let Some(old_score) = &e.value_map.remove(v) {
                if let Some(v_set) = e.score_map.get_mut(old_score) {
                    v_set.remove(v);
                }
            }
        }
    }
    #[inline]
    pub async fn insert(&self, req: InsrtRequest) {
        self.insert_local(req);

        let cur_id = **IDX;

        let mut v = vec![];
        for i in 0..CLUSTER_NUM {
            if i == cur_id {
                continue;
            }
            v.push(http_req::add(&self.client, &CLUSTER_URL[i], req.clone()));
        }
        futures::future::join_all(v).await;*/
    }

    #[inline]
    pub fn insert_local(&self, req: InsrtRequest) {
        let map = &self.map_arr[shard_idx(&req.key)];
        map.insert(req.key, req.value);
    }

    #[inline]
    pub async fn del(&self, k: &String) {
        self.map_arr[shard_idx(&k)].remove(k);

        /*let cur_id = **IDX;

        let mut v = vec![];
        for i in 0..CLUSTER_NUM {
            if i == cur_id {
                continue;
            }
            v.push(http_req::del(&self.client, &CLUSTER_URL[i], k));
        }
        futures::future::join_all(v).await;*/
    }

    #[inline]
    pub fn del2(&self, k: &String) {
        self.map_arr[shard_idx(&k)].remove(k);
    }

    #[inline]
    pub async fn get(&self, k: &String) -> Option<String> {
        self.local_get(k)
    }

    pub fn local_get(&self, k: &String) -> Option<String> {
        (&self.map_arr[shard_idx(&k)]).get(k)
            //.map(|e| e.value().to_string())
            .map(|e| e.to_string())
    }
    #[inline]
    pub async fn list(&self, keys: Vec<String>) -> Vec<InsrtRequest> {
        let mut ret = vec![];
        for k in keys.iter() {
            match self.local_get(k) {
                None => {}
                Some(v) => {
                    ret.push(InsrtRequest::new(k.clone(), v));
                }
            }
        }


        ret
    }

    #[inline]
    pub async fn batch_insert(&self, reqs: Vec<InsrtRequest>) {
        self.batch_insert2(reqs.clone());

        let cur_id = **IDX;

        let mut v = vec![];
        for i in 0..CLUSTER_NUM {
            if i == cur_id {
                continue;
            }
            v.push(http_req::batch(&self.client, &CLUSTER_URL[i], &reqs));
        }
        futures::future::join_all(v).await;
    }

    pub fn batch_insert2(&self, reqs: Vec<InsrtRequest>) {
        for req in reqs.iter() {
            self.insert_local(req.clone());
        }
    }
}

#[test]
fn test_load_level_db() {
    let kv = Store::new();
    kv.load_from_file();
}