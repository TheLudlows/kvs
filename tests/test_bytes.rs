use std::str::from_utf8;
use bytes::Bytes;
use lifeguard::{Pool, pool, StartingSize};

#[test]
fn test_bytes() {
    let bytes = Bytes::from("abc".to_string());
    let ss = from_utf8(&bytes).unwrap();
    println!("{}", ss);


    let pool : Pool<String> = pool().with(StartingSize(10)).build();
    let mut string = pool.new_from("cat");
}