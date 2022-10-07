mod model;

use std::{thread};
use std::net::SocketAddr;
use std::sync::Arc;
use log::info;
use warp::{Filter, Reply};
use log4rs::init_file;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use warp::http::StatusCode;
use warp::reject::InvalidQuery;
use warp::reply::Response;

pub use model::*;

use crate::request::*;
use crate::cluster::*;
use crate::evn::read_port;
use crate::store::*;


//#[global_allocator]
//static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;


//#[tokio::main(worker_threads = 20)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kv = Arc::new(Store::new());
    init_file("./log4rs.yml", Default::default())?;
    load_cluster_from_disk();
    kv.load_from_file();
    // load cluster info
    let kv = warp::any().map(move || kv.clone());

    let update = warp::path("updateCluster")
        .and(warp::body::json())
        .map(|c: Cluster| {
            set_cluster(c);
            return format!("ok");
        });


    let init_route = warp::path("init")
        .and(kv.clone())
        .map(|kv: Arc<Store>| {
            return kv.load_from_file();
        });

    let query = warp::path("query")
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .and_then(|k, kv: Arc<Store>| async move {
            match kv.get(&k).await {
                None => {
                    Err(warp::reject::not_found())
                }
                Some(v) => {
                    Ok(v)
                }
            }
        });


    let add = warp::path("add")
        .and(warp::body::json())
        .and(kv.clone())
        .then(|req: InsrtRequest, kv: Arc<Store>| async move {
            kv.insert(req).await;
            warp::reply::reply()
        });
    let add2 = warp::path("add2")
        .and(warp::body::json())
        .and(kv.clone())
        .then(|req: InsrtRequest, kv: Arc<Store>| async move {
            kv.insert_local(req);
            warp::reply::reply()
        });

    let del = warp::path("del")
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .then(|k, kv: Arc<Store>| async move {
            //info!("del key{:?}", k);
            kv.del(&k).await;
            return warp::reply::reply();
        });

    let del2 = warp::path("del2")
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .then(|k, kv: Arc<Store>| async move {
            //info!("del key{:?}", k);
            kv.del2(&k);
            return warp::reply::reply();
        });

    let list = warp::path("list")
        .and(warp::body::json())
        .and(kv.clone())
        .and_then(|keys: Vec<String>, kv: Arc<Store>| async move {
            let res = &kv.list(keys).await;
            if res.is_empty() {
                Err(warp::reject::not_found())
            } else {
                Ok(warp::reply::json(res))
            }
        });

    let batch = warp::path("batch")
        .and(warp::body::json())
        .and(kv.clone())
        .then(|vs: Vec<InsrtRequest>, kv: Arc<Store>| async move {
            //info!("{:?}", vs);
            kv.batch_insert(vs).await;
            return warp::reply();
        });

    let batch2 = warp::path("batch2")
        .and(warp::body::json())
        .and(kv.clone())
        .then(|vs: Vec<InsrtRequest>, kv: Arc<Store>| async move {
            //info!("{:?}", vs);
            kv.batch_insert2(vs);
            return warp::reply();
        });

    let zadd = warp::path("zadd")
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and(kv.clone())
        .then(|k: String, v: ScoreValue, kv: Arc<Store>| async move {
            //info!("{:?}{:?}",k, v);
            kv.zset_insert(k, v).await;
            return warp::reply();
        });

    let zadd2 = warp::path("zadd2")
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and(kv.clone())
        .then(|k: String, v: ScoreValue, kv: Arc<Store>| async move {
            //info!("{:?}{:?}",k, v);
            kv.do_insert(k, v);
            return warp::reply();
        });

    let zrange = warp::path("zrange")
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and(kv.clone())
        .and_then(|k: String, range: ScoreRange, kv: Arc<Store>| async move {
            let res = &kv.range(&k, range);
            if res.is_empty() {
                Err(warp::reject::not_found())
            } else {
                Ok(warp::reply::json(res))
            }
        });


    let zrmv = warp::path("zrmv")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .then(|k: String, v: String, kv: Arc<Store>| async move {
            //info!("{:?}{:?}",k, v);
            kv.remove(&k, &v).await;
            return warp::reply();
        });

    let zrmv2 = warp::path("zrmv2")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .then(|k: String, v: String, kv: Arc<Store>| async move {
            //info!("{:?}{:?}",k, v);
            kv.remove2(&k, &v);
            return warp::reply();
        });

    let apis = init_route.or(query).or(add).or(del)
        .or(list).or(batch).or(zadd).or(zrange).or(add2).or(zadd2).or(batch2).or(del2).or(zrmv2)
        .or(zrmv).or(update);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();


    let port = read_port();
    let address: SocketAddr = (String::from("0.0.0.0:") + &port).parse().unwrap();

    info!("rust server started at {}", address);
    let (_, server) = warp::serve(apis)
        .unstable_pipeline()
        .bind_with_graceful_shutdown(address, async move {
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
