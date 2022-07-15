use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};
use chrono::Local;
use dashmap::DashMap;
use flatbuffers::Push;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use tokio::time;

use kvs::model::request::*;


static ADD_COUNT: i32 = 10100;

static ADD_THREAD_NUM: usize = 8;

static DEL_COUNT: i32 = 100;

static QUERY_THREAD_NUM: usize = 8;

static DEL_START: i32 = 10000;

static ZADD_COUNT: i32 = 1000;

static ZADD_SUB_COUNT: i32 = 10;

static ZADD_THREAD_NUM: usize = 8;

static ZRMV_COUNT: i32 = 10;


#[test]
fn test_main() {
    let start = Local::now().timestamp_millis();
    let client = Client::new();
    test_init(client.clone());
    test_add(client.clone());
    test_query(client.clone());
    test_list(client.clone());
    test_batch(client.clone());
    test_del(client.clone());
    test_zadd(client.clone());
    test_range(client.clone());
    test_rmv(client.clone());
    let end = Local::now().timestamp_millis();
    println!("total cost :{}", (end - start))
}

async fn test_init(client: Client) {
    let res = client
        .get("http://localhost:8080/init")
        .header("Accept", "text/plain")
        .timeout(Duration::from_secs(3))
        .send()
        .unwrap()
        .text()
        .unwrap();
    assert_eq!(res, "ok");
}

fn test_add(client: Client) {
    let mut js = Vec::with_capacity(ADD_THREAD_NUM);
    let count = Arc::new(AtomicI32::from(ADD_COUNT));
    for _ in 0..ADD_THREAD_NUM {
        let client = client.clone();
        let count = count.clone();
        js.push(thread::spawn(move || {
            let mut i;
            loop {
                i = count.fetch_sub(1, Ordering::Release);
                if i < 0 {
                    break;
                }
                let req = InsrtRequest::new("key".to_string() + &i.to_string(), "val".to_string() + &i.to_string());
                let rep = client.post("http://localhost:8080/add")
                    .json(&req)
                    .send()
                    .unwrap();
                assert_eq!(rep.status(), StatusCode::OK);
            }
        }));
    }
    join_all(js);
}

fn test_query(client: Client) {
    let mut js = Vec::with_capacity(QUERY_THREAD_NUM);
    let count = Arc::new(AtomicI32::from(ADD_COUNT));
    for _ in 0..QUERY_THREAD_NUM {
        let client = client.clone();
        let count = count.clone();
        js.push(thread::spawn(move || {
            let mut i;
            loop {
                i = count.fetch_sub(1, Ordering::Release);
                if i < 0 {
                    break;
                }
                let rep = client.get(String::from("http://localhost:8080/query/key") + i.to_string().as_str())
                    .send()
                    .unwrap()
                    .text()
                    .unwrap();
                assert_eq!(rep, String::from("val") + i.to_string().as_str());
            }
        }));
    }
    join_all(js);
}

fn test_list(client: Client) {
    let mut js = Vec::with_capacity(QUERY_THREAD_NUM);
    let count = Arc::new(AtomicI32::from(0));
    for _ in 0..QUERY_THREAD_NUM {
        let client = client.clone();
        let count = count.clone();
        js.push(thread::spawn(move || {
            let mut n;
            loop {
                n = count.fetch_add(10, Ordering::Release);
                if n >= ADD_COUNT {
                    break;
                }
                let mut v = vec![];
                for i in n..n + 10 {
                    v.push(String::from("key") + i.to_string().as_str());
                }
                let rep: Vec<InsrtRequest> = client.post(String::from("http://localhost:8080/list"))
                    .json(&v)
                    .send()
                    .unwrap()
                    .json()
                    .unwrap();
                assert_eq!(rep.len(), v.len());
            }
        }));
    }
    join_all(js);
}

fn test_batch(client: Client) {
    let mut js = Vec::with_capacity(ADD_THREAD_NUM);
    let count = Arc::new(AtomicI32::from(2 * ADD_COUNT));
    for _ in 0..QUERY_THREAD_NUM {
        let client = client.clone();
        let count = count.clone();
        js.push(thread::spawn(move || {
            let mut n;
            loop {
                n = count.fetch_sub(10, Ordering::Release);
                if n < ADD_COUNT {
                    break;
                }
                let mut v = vec![];
                for i in n..n + 10 {
                    v.push(InsrtRequest::new("key".to_string() + &i.to_string(), "val".to_string() + &i.to_string()));
                }
                let rep = client.post(String::from("http://localhost:8080/batch"))
                    .json(&v)
                    .send()
                    .unwrap();
                assert_eq!(rep.status(), StatusCode::OK);
            }
        }));
    }
    join_all(js);
}


fn test_del(client: Client) {
    for i in DEL_START..ADD_COUNT {
        let rep = client.get(String::from("http://localhost:8080/del/key") + i.to_string().as_str())
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::OK);
    }
    for i in DEL_START..ADD_COUNT {
        let rep = client.get(String::from("http://localhost:8080/query/key") + i.to_string().as_str())
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::OK);
    }
}

fn test_zadd(client: Client) {
    let mut js = Vec::with_capacity(ZADD_THREAD_NUM);
    let count = Arc::new(AtomicI32::from(ZADD_COUNT));
    for _ in 0..ZADD_THREAD_NUM {
        let client = client.clone();
        let count = count.clone();
        js.push(thread::spawn(move || {
            let mut n;
            loop {
                n = count.fetch_sub(1, Ordering::Release);
                if n < 0 {
                    break;
                }
                for j in 0..ZADD_SUB_COUNT {
                    let score = ScoreValue::new(j as u32, j.to_string());
                    let rep = client.post(String::from("http://localhost:8080/zadd/") + &String::from("key") + &n.to_string())
                        .json(&score)
                        .send()
                        .unwrap();
                    assert_eq!(rep.status(), StatusCode::OK);
                }
            }
        }));
    }
    join_all(js);
}

fn test_range(client: Client) {
    let mut js = Vec::with_capacity(ZADD_THREAD_NUM);
    let count = Arc::new(AtomicI32::from(ZADD_COUNT));
    for _ in 0..ZADD_THREAD_NUM {
        let client = client.clone();
        let count = count.clone();
        js.push(thread::spawn(move || {
            let mut n;
            loop {
                n = count.fetch_sub(1, Ordering::Release);
                if n < 0 {
                    break;
                }
                let score = ScoreRange::new(0, (ZADD_SUB_COUNT / 2) as u32);
                let rep: Vec<ScoreValue> = client.post(String::from("http://localhost:8080/zrange/") + &String::from("key") + &n.to_string())
                    .json(&score)
                    .send()
                    .unwrap()
                    .json()
                    .unwrap();
                assert_eq!(rep.len(), (ZADD_SUB_COUNT / 2 + 1) as usize);
            }
        }));
    }
    join_all(js);
}

fn test_rmv(client: Client) {
    for i in 0..ZRMV_COUNT {
        let rep = client.get(String::from("http://localhost:8080/zrmv/") + &String::from("key") + &i.to_string() + "/0")
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::OK);
    }

    for i in 0..ZRMV_COUNT {
        let score = ScoreRange::new(0, (ZADD_SUB_COUNT / 2) as u32);
        let rep: Vec<ScoreValue> = client.post(String::from("http://localhost:8080/zrange/") + &String::from("key") + &i.to_string())
            .json(&score)
            .send()
            .unwrap()
            .json()
            .unwrap();
        assert_eq!(rep.len(), (ZADD_SUB_COUNT / 2) as usize);
    }
}

pub fn join_all(js: Vec<JoinHandle<()>>) {
    for j in js {
        j.join().unwrap();
    }
}

#[test]
fn test_map() {
    let map = DashMap::with_capacity(4);
    for i in 0..10 {
        map.insert(i,i);
    }
    map.remove(&0);
    println!("{:?}", map);
}
