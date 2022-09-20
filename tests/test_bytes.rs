use std::str::from_utf8;
use bytes::Bytes;

#[test]
fn test_bytes() {
    let bytes = Bytes::from("abc".to_string());
    let ss = from_utf8(&bytes).unwrap();
    println!("{}", ss);


    bstr::BStr::new()
}