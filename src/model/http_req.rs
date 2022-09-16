use reqwest::{Client, StatusCode};
use crate::model::request::{Cluster, InsrtRequest, ScoreRange, ScoreValue};


pub async fn update_cluster(client: &Client, host: &String, cluster: Cluster) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/updateCluster")
        .json(&cluster)
        .send()
        .await?;
    println!("{:?}", _rep);
    Ok(())
}

pub async fn init(client: &Client, host: &String) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/init")
        .send()
        .await?;
    Ok(())
}


pub async fn add(client: &Client, host: &String, req: InsrtRequest) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/add")
        .json(&req)
        .send()
        .await?;
    Ok(())
}

pub async fn query(client: &Client, host: &String, key: &String) -> Result<Option<String>, reqwest::Error> {

    let resp = client.get(String::from(host) + "/query/" + key)
        .send()
        .await?;
    if resp.status() == StatusCode::NOT_FOUND{
        return Ok(None);
    }
    Ok(Some(resp.text().await?))
}

pub async fn list(client: &Client, host: &String, keys: &Vec<String>) -> Result<Vec<InsrtRequest>, reqwest::Error> {
    let rep: Vec<InsrtRequest> = client.post(String::from(host) + "/list")
        .json(&keys)
        .send()
        .await?
        .json()
        .await?;
    Ok(rep)
}

pub async fn batch(client: &Client, host: &String, req: Vec<InsrtRequest>) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/batch")
        .json(&req)
        .send()
        .await?;
    Ok(())
}


pub async fn del(client: &Client, host: &String, key: &String) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/del/" + key)
        .send()
        .await?;
    Ok(())
}

pub async fn zadd(client: &Client, host: &String, key: String, sv: ScoreValue) -> Result<(), reqwest::Error> {
    let _rep = client.post(String::from(host) + "/zadd/" + &key)
        .json(&sv)
        .send()
        .await?;
    Ok(())
}

pub async fn range(client: &Client, host: &String, key: &String, range: ScoreRange) -> Result<Vec<ScoreValue>, reqwest::Error> {
    let rep: Vec<ScoreValue> = client.post(String::from(host) + "/zrange/" + key)
        .json(&range)
        .send()
        .await?
        .json()
        .await?;
    Ok(rep)
}

pub async fn rmv(client: &Client, host: &String, key: &String, val: &String) -> Result<(), reqwest::Error> {
    let _rep = client.get(String::from(host) + "/zrmv/" + key + "/" + val)
        .send()
        .await?;
    Ok(())
}
