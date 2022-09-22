use std::collections::BTreeMap;
use skiplist::{SkipList, SkipMap};
use kvs::model::request::{ScoreRange, ScoreValue};
use kvs::model::store::Store;

#[test]
fn test_zset() {

    let store = Store::new();

    store.do_insert("key1".to_string(), ScoreValue::new(1, "a".to_string()));

    let vec = store.range_Local(&"key1".to_string(), ScoreRange::new(1,1));
    println!("{:?}", vec);

    store.do_insert("key1".to_string(), ScoreValue::new(1, "b".to_string()));

    let vec = store.range_Local(&"key1".to_string(), ScoreRange::new(1,1));
    println!("{:?}", vec);

    store.do_insert("key1".to_string(), ScoreValue::new(2, "b".to_string()));

    let vec = store.range_Local(&"key1".to_string(), ScoreRange::new(1,2));
    println!("{:?}", vec);

}