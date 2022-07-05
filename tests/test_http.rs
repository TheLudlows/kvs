use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use dashmap::DashMap;
use http::StatusCode;
use reqwest::blocking::Client;

use kvs::model::request::*;


static ADD_COUNT: i32 = 1000;

static DEL_COUNT: i32 = 100;

static ZADD_COUNT: i32 = 1000;

static ZADD_SUB_COUNT: i32 = 10;

static ZRMV_COUNT: i32 = 10;


#[test]
fn test_main() {
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
    for i in 0..ADD_COUNT {
        let req = InsrtRequest::new("key".to_string() + &i.to_string(), "val".to_string() + &i.to_string());
        let rep = client.post("http://localhost:8080/add")
            .json(&req)
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::OK);
    }
}

fn test_query(client: Client) {
    for i in 0..ADD_COUNT {
        let rep = client.get(String::from("http://localhost:8080/query/key") + i.to_string().as_str())
            .send()
            .unwrap()
            .text()
            .unwrap();
        assert_eq!(rep, String::from("val") + i.to_string().as_str());
    }
}

fn test_list(client: Client) {
    let mut n = 0;
    while n < ADD_COUNT {
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
        assert_eq!(rep.len(), 10);
        n += 10;
    }
}

fn test_batch(client: Client) {
    let mut n = ADD_COUNT;
    while n < ADD_COUNT * 2 {
        let mut v = vec![];
        for i in n..n + 10 {
            v.push(InsrtRequest::new("key".to_string() + &i.to_string(), "val".to_string() + &i.to_string()));
        }
        let rep = client.post(String::from("http://localhost:8080/batch"))
            .json(&v)
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::OK);
        n += 10;
    }
}


fn test_del(client: Client) {
    for i in 0..DEL_COUNT {
        let rep = client.get(String::from("http://localhost:8080/del/key") + i.to_string().as_str())
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::OK);
    }
    for i in 0..DEL_COUNT {
        let rep = client.get(String::from("http://localhost:8080/query/key") + i.to_string().as_str())
            .send()
            .unwrap();
        assert_eq!(rep.status(), StatusCode::NOT_FOUND);
    }
}

fn test_zadd(client: Client) {
    for i in 0..ZADD_COUNT {
        for j in 0..ZADD_SUB_COUNT {
            let score = ScoreValue::new(j as u32, j.to_string());
            let rep = client.post(String::from("http://localhost:8080/zadd/") + &String::from("key") + &i.to_string())
                .json(&score)
                .send()
                .unwrap();
            assert_eq!(rep.status(), StatusCode::OK);
        }
    }
}

fn test_range(client: Client) {
    for i in 0..ZADD_COUNT {
        let score = ScoreRange::new(0, (ZADD_SUB_COUNT / 2) as u32);
        let rep: Vec<ScoreValue> = client.post(String::from("http://localhost:8080/zrange/") + &String::from("key") + &i.to_string())
            .json(&score)
            .send()
            .unwrap()
            .json()
            .unwrap();
        assert_eq!(rep.len(), (ZADD_SUB_COUNT / 2 + 1) as usize);
    }
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


#[test]
fn test_alter() {
    let map = Arc::new(DashMap::new());
    let mut js = Vec::with_capacity(8);
    for i in 0..8 {
        let map = map.clone();
        js.push(thread::spawn(move || {
            let mut count = 10000;
            while count >= 0 {
                map.entry(count.to_string()).or_insert_with(|| Vec::new());
                map.alter(&count.to_string(), |k, mut v| {
                    v.push(i.to_string());
                    return v;
                });
                count -= 1;
            }
        }));
    }

    for j in js {
        j.join().unwrap();
    }

    js = Vec::new();
    for i in 0..8 {
        let map = map.clone();
        js.push(thread::spawn(move || {
            let mut count = 10000;
            while count >= 0 {
                match map.get(&count.to_string()) {
                    None => {
                        println!("count {} empty", count)
                    }
                    Some(v) => {
                        if v.value().len() != 8 {
                            println!("count {} len err", count)
                        }
                    }
                }
                count -= 1;
            }
        }));
    }
    for j in js {
        j.join().unwrap();
    }
}

