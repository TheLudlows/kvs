use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;
use fs_extra::dir::{ls, DirEntryAttr, LsResult};
use std::collections::HashSet;
use std::fs::{create_dir, create_dir_all};
use std::path::PathBuf;
use fs_extra::dir;

#[test]
fn test_cp_dir() {
    let mut options = CopyOptions::new();
    options.skip_exist = true;

    copy("/tmp/data1", "/tmp/data/", &options);



    let mut config = HashSet::new();
    config.insert(DirEntryAttr::IsDir);


    let mut pb = PathBuf::from("/Users/liuchao/data");

     pb.push("abc");


    println!("{:?}", pb);
    if !pb.exists() {
        dir::create(pb, true).unwrap();
    }
}