use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use chrono::Local;
use reqwest::{Client};
use kvs::model::http_req;

use kvs::model::request::*;


static ADD_COUNT: i32 = 300;

static DEL_COUNT: i32 = 10;

static ZADD_COUNT: i32 = 300;

static ZADD_SUB_COUNT: i32 = 10;
static ZRMV_COUNT: i32 = 10;


#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

lazy_static! {
    static ref CLUSTER_URLS: Vec<String> =  {
      /** let v = vec![
            String::from("http://172.16.0.158:8080"),
            String::from("http://172.16.0.164:8080"),
            String::from("http://172.16.0.187:8080"),
        ];*/
           let v = vec![
            String::from("http://localhost:8080"),
            String::from("http://localhost:8081"),
            String::from("http://localhost:8082"),
        ];
        v
    };

    static ref CLUSTERS: Vec<String> =  {
       /* let v = vec![
            String::from("172.16.0.158"),
            String::from("172.16.0.164"),
            String::from("172.16.0.187"),
        ];*/
         let v = vec![
            String::from("localhost:8080"),
            String::from("localhost:8081"),
            String::from("localhost:8082"),
        ];
        v
    };
    static ref client:Client = Client::new();
}

#[tokio::test]
pub async fn test_main() -> Result<(), reqwest::Error> {
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

/**
 172.16.0.158
172.16.0.164
 172.16.0.187
 */

/**
#[tokio::test]
pub async  fn test_single() {
    let cluster_url = vec![String::from("localhost"), String::from("localhost"), String::from("localhost")];
    let cluster = Cluster{
        hosts: cluster_url,
        index: 1
    };
    let res = http_req::update_cluster(&client, &String::from("http://60.205.189.30:8080"), cluster).await;
    println!("{:?}", res);
    let res = http_req::init(&client, &String::from("http://60.205.189.30:8080")).await;
    println!("{:?}", res);

    let res = http_req::query(&client, &String::from("http://60.205.189.30:8080"), &String::from("k1")).await;
    println!("{:?}", res);
}
*/
#[tokio::test]
async fn test_restart() -> Result<(), reqwest::Error> {
    let start = Local::now().timestamp_millis();
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

pub async fn test_update_cluster() -> Result<(), reqwest::Error> {
    for i in 1..=3 {
        let cluster = Cluster {
            hosts: CLUSTERS.clone(),
            index: i,
        };
        let res = http_req::update_cluster(&client, &CLUSTER_URLS[i - 1], cluster).await?;
        assert_eq!(res, "ok");
    }
    Ok(())
}


pub async fn test_init() -> Result<(), reqwest::Error> {
    for s in CLUSTER_URLS.iter() {
        http_req::init(&client, s).await?;
    }
    Ok(())
}

pub async fn test_add() -> Result<(), reqwest::Error> {
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

pub async fn test_query() -> Result<(), reqwest::Error> {
    for host in CLUSTER_URLS.iter() {
        for i in 0..ADD_COUNT {
            let res = http_req::query(&client, host, &(String::from("key") + &i.to_string())).await?.unwrap();
            assert_eq!(res, String::from("val") + i.to_string().as_str());
        }
    }
    Ok(())
}

pub async fn test_list() -> Result<(), reqwest::Error> {
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

pub async fn test_batch() -> Result<(), reqwest::Error> {
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

pub async fn test_del() -> Result<(), reqwest::Error> {
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

pub async fn test_zadd() -> Result<(), reqwest::Error> {
    let mut n = 0;
    for host in CLUSTER_URLS.iter() {
        for i in n..n + ZADD_COUNT / 3 {
            for j in 0..ZADD_SUB_COUNT {
                let score = ScoreValue::new(j as u32, j.to_string());
                let res = http_req::zadd(&client, host, String::from("key") + &i.to_string(), score).await;
                assert!(res.is_ok());
            }
        }
        n += ZADD_COUNT / 3;
    }
    Ok(())
}

pub async fn test_range() -> Result<(), reqwest::Error> {
    for host in CLUSTER_URLS.iter() {
        for i in 0..ZADD_COUNT {
            let score = ScoreRange::new(0, (ZADD_SUB_COUNT / 2) as u32);
            let res = http_req::range(&client, host, &(String::from("key") + &i.to_string()), score).await?;
            if res.len() != (ZADD_SUB_COUNT / 2 + 1) as usize {
                println!("{}", &(String::from("key") + &i.to_string()))
            }
            assert_eq!(res.len(), (ZADD_SUB_COUNT / 2 + 1) as usize);

            let score = ScoreRange::new(0, (ZADD_SUB_COUNT) as u32);
            let res = http_req::range(&client, host, &(String::from("key") + &i.to_string()), score).await?;
            if res.len() != (ZADD_SUB_COUNT) as usize {
                println!("{}", &(String::from("key") + &i.to_string()))
            }
            assert_eq!(res.len(), (ZADD_SUB_COUNT) as usize);
        }
    }
    Ok(())
}

pub async fn test_rmv() -> Result<(), reqwest::Error> {
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
