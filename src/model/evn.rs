use std::env;

#[cfg(target_os = "macos")]
pub static BASE_PATH: &str = "/Users/liuchao/data";
#[cfg(target_os = "macos")]
pub static data_files: [&'static str; 3] = ["/Users/liuchao/data1", "/Users/liuchao/data2", "/Users/liuchao/data3"];


#[cfg(target_os = "linux")]
pub static BASE_PATH: &str = "/conf";
#[cfg(target_os = "linux")]
pub static data_files: [&'static str; 3] = ["/data/data1", "/data/data2", "/data/data3"];

pub static CLUSTER_FILE: &str = "cluster";

pub const SHARD_NUM: usize = 8;

pub const DEFAULT_SIZE: usize = 1024 * 8;

pub const DEFAULT_KV_SIZE: usize = 1024 * 8;

pub const CLUSTER_NUM: usize = 3;

pub const TOTAL_SLOTS:usize = 3000;

pub const SUB_SLOTS:usize = TOTAL_SLOTS/3;

pub const MORE_CACHE:usize = 0;

#[inline]
pub fn shard_idx(s: &String) -> usize {
    if s.len() == 0 {
        return 0;
    }
    //fasthash::xx::hash32(s) as usize % SHARD_NUM
    fasthash::xx::hash32(s) as usize % SHARD_NUM
}

#[inline]
pub fn cluster_idx(s: &str) -> usize {
    if s.len() == 0 {
        return 0;
    }
    let hash_code = fasthash::xx::hash32(s) as usize % TOTAL_SLOTS;
    let mut idx = 0;

    // 0=..1200
    if  hash_code < SUB_SLOTS + MORE_CACHE {
        idx |= 1;
    }
    // 1000=..2200
    if hash_code >= SUB_SLOTS && hash_code < SUB_SLOTS*2 + MORE_CACHE {
        idx |=2;
    }

    //2000..=2999 + 0..200
    if hash_code >= SUB_SLOTS*2 || hash_code < MORE_CACHE {
        idx |=4;
    }
    idx
}


pub fn read_port() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return String::from("8080");
    } else {
        return args[1].clone();
    }
}
