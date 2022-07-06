mod model;


use std::{collections::HashMap, thread};
use std::sync::Arc;
use log::info;
use warp::Filter;
use log4rs::init_file;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use crate::model::*;
use crate::model::request::*;

pub use model::*;
use crate::evn::SHARD_NUM;
use crate::store::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kv = Arc::new(Kv::new());
    let zset = Arc::new(ZSet::new());

    kv.load_from_file();
    let kv = warp::any().map(move || kv.clone());
    let zset = warp::any().map(move || zset.clone());


    init_file("./log4rs.yml", Default::default())?;
    let init_route = warp::get().and(warp::path("init")).map(|| {
        return format!("ok");
    });
    let query = warp::path("query")
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .and_then(|k, kv: Arc<Kv>| {
            //info!("query {:?}", k);
            let resp = match kv.get(&k) {
                None => {
                    Err(warp::reject::not_found())
                }
                Some(v) => {
                    Ok(v)
                }
            };
            async move { resp }
        });


    let add = warp::path("add")
        .and(warp::body::json())
        .and(kv.clone())
        .map(|req: InsrtRequest, kv: Arc<Kv>| {
            kv.insert(req.key, req.value);
            return warp::reply::reply();
        });

    let del = warp::path("del")
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .map(|k, kv: Arc<Kv>| {
            //info!("del key{:?}", k);
            kv.del(k);
            return warp::reply::reply();
        });

    let list = warp::path("list")
        .and(warp::body::json())
        .and(kv.clone())
        .map(|keys: Vec<String>, kv: Arc<Kv>| {
            warp::reply::json(&kv.list(keys))
        });

    let batch = warp::path("batch")
        .and(warp::body::json())
        .and(kv.clone())
        .map(|vs: Vec<InsrtRequest>, kv: Arc<Kv>| {
            //info!("{:?}", vs);
            kv.batch_insert(vs);
            return warp::reply();
        });


    let zadd = warp::path("zadd")
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and(zset.clone())
        .map(|k: String, v: ScoreValue, zset: Arc<ZSet>| {
            //info!("{:?}{:?}",k, v);
            zset.insert(k, v);
            return warp::reply();
        });

    let zrange = warp::path("zrange")
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and(zset.clone())
        .and_then(|k: String, range: ScoreRange, zset: Arc<ZSet>| {
            let res = zset.range(&k, range.clone());
            //info!("zrange{:?} {:?} {:?}", k, range, res);
            async move {
                if res.len() == 0 {
                    Err(warp::reject::not_found())
                } else {
                    Ok(warp::reply::json(&res))
                }
            }
        });


    let zrmv = warp::path("zrmv")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(zset.clone())
        .map(|k: String, v: String, zset: Arc<ZSet>| {
            //info!("{:?}{:?}",k, v);
            zset.remove(&k, &v);
            return warp::reply();
        });

    let apis = init_route.or(query).or(add).or(del)
        .or(list).or(batch).or(zadd).or(zrange)
        .or(zrmv);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let (_, server) = warp::serve(apis)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async move {
            info!("rust server started");
            rx.recv().await;
            info!("rust servert receive close signal");
        });
    // Spawn the server into a runtime
    let thread = tokio::spawn(server);
    let mut signals = Signals::new(&[SIGTERM])?;
    let tx_signal = tx.clone();

    thread::spawn(move || {
        for _ in signals.forever() {
            let _ = tx_signal.send(()).unwrap();
            info!("start to close server");
            break;
        }
    });
    thread.await?;
    Ok(())
}