use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};
use chrono::Local;
use dashmap::DashMap;
use flatbuffers::Push;
use reqwest::{Client, StatusCode};
use tokio::time;
use kvs::model::http_req;

use kvs::model::request::*;


static ADD_COUNT: i32 = 300;

static DEL_COUNT: i32 = 10;

static ZADD_COUNT: i32 = 100;

static ZADD_SUB_COUNT: i32 = 10;
static ZRMV_COUNT: i32 = 10;


#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

lazy_static! {
    static ref CLUSTER_URLS: Vec<String> =  {
        let mut v = vec![
            String::from("http://localhost:8080"),
            String::from("http://localhost:8081"),
            String::from("http://localhost:8082"),
        ];
        v
    };

    static ref CLUSTERS: Vec<String> =  {
        let mut v = vec![
            String::from("localhost:8080"),
            String::from("localhost:8081"),
            String::from("localhost:8082"),
        ];
        v
    };
    static ref client:Client = Client::new();
}

#[tokio::test]
async fn test_main() -> Result<(), reqwest::Error> {
    let start = Local::now().timestamp_millis();
    test_update_cluster().await?;
    test_init().await?;
    test_add().await?;
    test_query().await?;
    test_list().await?;
    test_batch().await?;
    test_del().await?;
    test_zadd().await?;
    test_range().await?;
    test_rmv().await?;
    let end = Local::now().timestamp_millis();
    println!("total cost :{}", (end - start));
    Ok(())
}

async fn test_update_cluster() -> Result<(), reqwest::Error> {
    for i in 1..=3 {
        let cluster = Cluster {
            hosts: CLUSTERS.clone(),
            value: i,
        };
        http_req::update_cluster(&client, &CLUSTER_URLS[i - 1], cluster).await?;
    }
    Ok(())
}


async fn test_init() -> Result<(), reqwest::Error> {
    for s in CLUSTER_URLS.iter() {
        http_req::init(&client, s).await?;
    }
    Ok(())
}

async fn test_add() -> Result<(), reqwest::Error> {
    let mut n = 0;
    for host in CLUSTER_URLS.iter() {
        for i in n..n + ADD_COUNT / 3 {
            let req = InsrtRequest::new("key".to_string() + &i.to_string(), "val".to_string() + &i.to_string());
            let res = http_req::add(&client, host, req).await;
            assert!(res.is_ok());
        };
        n += ADD_COUNT / 3;
    }
    Ok(())
}

async fn test_query() -> Result<(), reqwest::Error> {
    let mut n = 0;
    for host in CLUSTER_URLS.iter() {
        for i in n..n + ADD_COUNT / 3 {
            let res = http_req::query(&client, host, &(String::from("key") + &i.to_string())).await?.unwrap();
            assert_eq!(res, String::from("val") + i.to_string().as_str());
        }
        n += ADD_COUNT / 3;
    }
    Ok(())
}

async fn test_list() -> Result<(), reqwest::Error> {
    for host in CLUSTER_URLS.iter() {
        let count = Arc::new(AtomicI32::from(0));
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
            let rep = http_req::list(&client, host, &v).await?;
            assert_eq!(rep.len(), v.len());
        }
    }
    Ok(())
}

async fn test_batch() -> Result<(), reqwest::Error> {
    let mut n = ADD_COUNT;
    for host in CLUSTER_URLS.iter() {
        let mut v = Vec::new();
        for i in n..n + 10 {
            v.push(InsrtRequest::new("key".to_string() + &i.to_string(), "val".to_string() + &i.to_string()));
        }
        let rep = http_req::batch(&client, host, v).await;
        assert!(rep.is_ok());
        n += 10;
    }
    Ok(())
}


async fn test_del() -> Result<(), reqwest::Error> {
    let mut n = DEL_COUNT;
    for host in CLUSTER_URLS.iter() {
        for i in n..n + 10 {
            http_req::del(&client, &host, &i.to_string()).await?;
        }
        n += 10;
    }

    let mut n = DEL_COUNT;
    for host in CLUSTER_URLS.iter() {
        for i in n..n + 10 {
            let rep = http_req::query(&client, host, &i.to_string()).await?;
            assert_eq!(rep, None);
        }
        n += 10;
    }
    Ok(())
}

async fn test_zadd() -> Result<(), reqwest::Error> {
    for host in CLUSTER_URLS.iter() {
        let count = Arc::new(AtomicI32::from(ZADD_COUNT));
        let mut n;
        loop {
            n = count.fetch_sub(1, Ordering::Release);
            if n < 0 {
                break;
            }
            for j in 0..ZADD_SUB_COUNT {
                let score = ScoreValue::new(j as u32, j.to_string());
                let res = http_req::zadd(&client, host, String::from("key") + &n.to_string(), score).await;
                assert!(res.is_ok());
            }
        }
    }
    Ok(())
}

async fn test_range() -> Result<(), reqwest::Error> {
    for host in CLUSTER_URLS.iter() {
        let count = Arc::new(AtomicI32::from(ZADD_COUNT));
        let mut n;
        loop {
            n = count.fetch_sub(1, Ordering::Release);
            if n < 0 {
                break;
            }
            let score = ScoreRange::new(0, (ZADD_SUB_COUNT / 2) as u32);
            let res = http_req::range(&client, host, &(String::from("key") + &n.to_string()), score).await?;
            assert_eq!(res.len(), (ZADD_SUB_COUNT / 2 + 1) as usize);
        }
    }
    Ok(())
}

async fn test_rmv() -> Result<(), reqwest::Error> {
    for host in CLUSTER_URLS.iter() {
        for i in 0..ZRMV_COUNT {
            http_req::rmv(&client, host, &(String::from("key") + &i.to_string()), &String::from("0")).await?;
        }
    }
    for host in CLUSTER_URLS.iter() {
        for i in 0..ZRMV_COUNT {
            let score = ScoreRange::new(0, (ZADD_SUB_COUNT / 2) as u32);
            let res = http_req::range(&client, host, &(String::from("key") + &i.to_string()), score).await?;
            assert_eq!(res.len(), (ZADD_SUB_COUNT / 2) as usize);
        }
    }
    Ok(())
}
