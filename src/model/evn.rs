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

#[inline]
pub fn shard_idx(s: &[u8]) -> usize {
    if s.len() == 0 {
        return 0;
    }
    //fasthash::xx::hash32(s) as usize % SHARD_NUM
    s[s.len() - 1] as usize % SHARD_NUM
}

#[inline]
pub fn cluster_idx(s: &[u8]) -> usize {
    if s.len() == 0 {
        return 0;
    }
    fasthash::xx::hash32(s) as usize % CLUSTER_NUM
}

pub fn read_port() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return String::from("8080");
    } else {
        return args[1].clone();
    }
}
