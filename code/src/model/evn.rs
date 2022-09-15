

#[cfg(target_os = "macos")]
pub static BASE_PATH: &str = "/Users/liuchao/data";
#[cfg(target_os = "macos")]
pub static data_files: [&'static str; 3] = ["/Users/liuchao/data1", "/Users/liuchao/data2", "/Users/liuchao/data3"];


#[cfg(target_os = "linux")]
pub static BASE_PATH: &str = "/data";
#[cfg(target_os = "linux")]
pub static data_files: [&'static str; 3] = ["/data1", "/data2", "/data3"];


pub static total_data_files: [&'static str; 3] = ["data1", "data2", "data3"];

pub static DATA_PATH: &str = "total_data";

pub static CLUSTER_FILE_PATH: &str = "cluster";

pub const SHARD_NUM: usize = 16;

pub const DEFAULT_SIZE: usize = 1024 * 16;

pub const DEFAULT_KV_SIZE: usize = 1024 * 32;

const CLUSTER_NUM: usize= 3;

#[inline]
pub fn shard_idx(s: &String) -> usize {
    if s.len() == 0 {
        return 0;
    }

    //fasthash::xx::hash32(s) as usize % SHARD_NUM

    s.as_bytes()[s.len() - 1] as usize % SHARD_NUM
}

#[inline]
pub fn cluster_idx(s: &String) -> usize {
    if s.len() == 0 {
        return 0;
    }
    fasthash::xx::hash32(s) as usize % CLUSTER_NUM
}

