use std::fs::{create_dir, OpenOptions};
use std::io::{ Read, Write};
use std::path::PathBuf;
`use std::sync::atomic::AtomicBool;
use lazy_static::lazy_static;
use log::info;
use crate::model::evn::{BASE_PATH, CLUSTER_FILE, read_port};
use crate::model::request::Cluster;


lazy_static! {
    pub static ref CLUSTER_URL:Vec<String> = {
        let v = vec![String::new(), String::new(), String::new()];
        v
    };

    pub static ref IDX: Box<usize> = Box::new(999);

}

pub static LOADED: AtomicBool = AtomicBool::new(false);



pub fn set_cluster(c: Cluster) {
    let ptr_idx = ((*IDX).as_ref() as *const usize) as *mut usize;
    unsafe {
        ptr_idx.write(c.index - 1);

        let mut ptr = CLUSTER_URL.as_ptr() as *mut String;
        for s in c.hosts.iter() {
            let mut host = String::from("http://");
            host.push_str(s);
            if !s.contains(":") {
                host.push_str(":8080");
            }
            let _old = ptr.replace(host);
            ptr = ptr.add(1);
        }
    }
    println!("set cluster {:?}", c);
    println!("cur node is {}", *IDX);
    println!("cluster is {:?}", *CLUSTER_URL);
    // write to file
    let file = conf_path();

    if file.exists() {
        return;
    }
    info!("crate conf file {:?}", file);

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(file).unwrap();
    f.write_all(serde_json::to_string(&c).unwrap().as_bytes()).unwrap();
}

pub fn load_cluster_from_disk() {
    let file = conf_path();
    if !file.exists() {
        return;
    }
    let mut f = OpenOptions::new()
        .read(true)
        .open(file).unwrap();
    let mut body = String::new();
    f.read_to_string(&mut body).unwrap();

    let cluster: Cluster = serde_json::from_str(&body).unwrap();
    set_cluster(cluster);
}

pub fn conf_path() -> PathBuf {
    let mut file = PathBuf::from(BASE_PATH);
    file = file.join(read_port());
    if !file.exists() {
        create_dir(&file).unwrap();
    }
    return file.join(CLUSTER_FILE);
}