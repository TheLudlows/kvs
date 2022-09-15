use std::num::NonZeroUsize;
use dashmap::RwLock;
use lru::LruCache;


#[test]
fn test_lru() {
    let mut cache = LruCache::new(NonZeroUsize::new(2).unwrap());

}