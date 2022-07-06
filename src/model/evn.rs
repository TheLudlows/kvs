

pub static LEVEL_DB_PATH: &str = "/Users/liuchao/data/";

pub static LEVEL_DB_ONLINE_PATH: &str = "/data";

pub const SHARD_NUM : usize = 32;


#[inline]
pub fn shard_idx(s: &String) -> usize{
    if s.len() == 0 {
        return 0;
    }
    (s.as_bytes()[s.len()-1] as usize % SHARD_NUM)
}
