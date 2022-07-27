mod model;
use std::thread;
use std::sync::Arc;
use bytes::Bytes;
use log::info;
use warp::{Filter};
use log4rs::init_file;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use warp::http::{Response};
use warp::hyper::Body;
use crate::model::request::*;

pub use model::*;
use crate::store::*;


//#[global_allocator]
//static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main(worker_threads = 16)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kv = Arc::new(Kv::new());
    let zset = Arc::new(ZSet::new());
    init_file("./log4rs.yml", Default::default())?;
    kv.load_from_file();
    let kv = warp::any().map(move || kv.clone());
    let zset = warp::any().map(move || zset.clone());


    let init_route = warp::get().and(warp::path("init")).map(|| {
        return format!("ok");
    });

    let query = warp::get().and(warp::path("query"))
        .and(warp::path::param::<String>())
        .and(kv.clone())
        .map(|k, kv: Arc<Kv>| {
           /* let resp = match kv.get(&k) {
                None => {
                    Err(warp::reject::not_found())
                }
                Some(v) => {
                    Ok(v)
                }
            };
            async move { resp }*/
            let k = Bytes::from(k);
            match kv.get(&k) {
                None => {
                    into_response(Bytes::from(""))
                }
                Some(v) => {
                    into_response(v)
                }
            }
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
            kv.del(&Bytes::from(k));
            return warp::reply::reply();
        });

    let list = warp::path("list")
        .and(warp::body::json())
        .and(kv.clone())
        .map(|keys: Vec<Bytes>, kv: Arc<Kv>| {
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
        .map(|k: String, range: ScoreRange, zset: Arc<ZSet>| {
            let res = zset.range(&k, range);
            //info!("zrange{:?} {:?} {:?}", k, range, res);
           /* async move {
                if res.len() == 0 {
                    Err(warp::reject::not_found())
                } else {
                    Ok(warp::reply::json(&res))
                }
            }*/
            warp::reply::json(&res)
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
        .unstable_pipeline()
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

fn into_response(bytes : Bytes) -> Response<Body> {
    Response::builder()
        .body(Body::from(bytes))
        .unwrap()
}