use log::{error, info};
use reqwest::{Client, StatusCode};
use crate::model::request::{Cluster, InsrtRequest, ScoreRange, ScoreValue};


pub async fn update_cluster(client: &Client, host: &str, cluster: Cluster) -> Result<String, reqwest::Error> {
    let _rep = client.post(String::from(host) + "/updateCluster")
        .json(&cluster)
        .send()
        .await?
        .text()
        .await?;
    Ok(_rep)
}

pub async fn init(client: &Client, host: &str) -> Result<String, reqwest::Error> {
    let _rep = client.get(String::from(host) + "/init")
        .send()
        .await?
        .text()
        .await?;
    Ok(_rep)
}


pub async fn add(client: &Client, host: &str, req: InsrtRequest) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/add2")
        .json(&req)
        .send()
        .await?;
    if _rep.status() != StatusCode::OK{
        error!("sys err")
    }
    return Ok(());
}

pub async fn query(client: &Client, host: &str, key: &String) -> Result<Option<String>, reqwest::Error> {
    let resp = client.get(String::from(host) + "/query/" + key)
        .send()
        .await?;
    if resp.status() == StatusCode::NOT_FOUND{
        return Ok(None);
    }
    Ok(Some(resp.text().await?))
}

pub async fn list(client: &Client, host: &str, keys: &Vec<String>) -> Result<Vec<InsrtRequest>, reqwest::Error> {
    let rep: Vec<InsrtRequest> = client.post(String::from(host) + "/list")
        .json(&keys)
        .send()
        .await?
        .json()
        .await?;
    Ok(rep)
}

pub async fn batch(client: &Client, host: &str, req: &Vec<InsrtRequest>) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/batch2")
        .json(req)
        .send()
        .await?;
    if !_rep.status().is_success() {
        println!("{:?}", _rep);
        panic!("batch post err")
    }
    Ok(())
}


pub async fn del(client: &Client, host: &str, key: &String) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/del2/" + key)
        .send()
        .await?;

    if !_rep.status().is_success() {
        panic!("del get err")
    }
    Ok(())
}

pub async fn zadd(client: &Client, host: &str, key: String, sv: ScoreValue) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/zadd2/" + &key)
        .json(&sv)
        .send()
        .await?;

    if !_rep.status().is_success() {
        panic!("zadd get err")
    }
    Ok(())
}

pub async fn range(client: &Client, host: &str, key: &String, range: ScoreRange) -> Result<Vec<ScoreValue>, reqwest::Error> {
    let rep: Vec<ScoreValue> = client.post(String::from(host) + "/zrange/" + key)
        .json(&range)
        .send()
        .await?
        .json()
        .await?;
    Ok(rep)
}

pub async fn rmv(client: &Client, host: &str, key: &String, val: &String) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/zrmv2/" + key + "/" + val)
        .send()
        .await?;

    if !_rep.status().is_success() {
        panic!("rmv get err")
    }
    Ok(())
}
