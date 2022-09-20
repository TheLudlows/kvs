use reqwest::{Client, StatusCode};
use smol_str::SmolStr;
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

pub async fn init(client: &Client, host: &str) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/init")
        .send()
        .await?;
    Ok(())
}


pub async fn add(client: &Client, host: &str, req: InsrtRequest) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/add")
        .json(&req)
        .send()
        .await?;
    if _rep.status() == StatusCode::OK{
        return Ok(());
    } else {
        panic!("err");
    }
}

pub async fn query(client: &Client, host: &str, key: &str) -> Result<Option<SmolStr>, reqwest::Error> {

    let resp = client.get(String::from(host) + "/query/" + key)
        .send()
        .await?;
    if resp.status() == StatusCode::NOT_FOUND{
        return Ok(None);
    }
    Ok(Some(SmolStr::new(resp.text().await?)))
}

pub async fn list(client: &Client, host: &str, keys: &Vec<SmolStr>) -> Result<Vec<InsrtRequest>, reqwest::Error> {
    let rep: Vec<InsrtRequest> = client.post(String::from(host) + "/list")
        .json(&keys)
        .send()
        .await?
        .json()
        .await?;
    Ok(rep)
}

pub async fn batch(client: &Client, host: &str, req: Vec<InsrtRequest>) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/batch")
        .json(&req)
        .send()
        .await?;
    Ok(())
}


pub async fn del(client: &Client, host: &String, key: &str) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/del/" + key)
        .send()
        .await?;
    Ok(())
}

pub async fn zadd(client: &Client, host: &String, key: &str, sv: ScoreValue) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/zadd/" + key)
        .json(&sv)
        .send()
        .await?;
    Ok(())
}

pub async fn range(client: &Client, host: &String, key: &str, range: ScoreRange) -> Result<Vec<ScoreValue>, reqwest::Error> {
    let rep: Vec<ScoreValue> = client.post(String::from(host) + "/zrange/" + key)
        .json(&range)
        .send()
        .await?
        .json()
        .await?;
    Ok(rep)
}

pub async fn rmv(client: &Client, host: &String, key: &str, val: &str) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/zrmv/" + key + "/" + val)
        .send()
        .await?;
    Ok(())
}
