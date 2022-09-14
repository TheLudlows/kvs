
pub static LEVEL_DB_PATH: &str = "/Users/liuchao/data/";

pub static LEVEL_DB_ONLINE_PATH: &str = "/data";

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

