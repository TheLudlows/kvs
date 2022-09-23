use kvs::model::evn::cluster_idx;

#[test]
fn test() {
    let idx = cluster_idx("a");

    println!("{}", idx);

    assert_ne!(idx & (1 << 1), 0);

    let idx = cluster_idx("b");

    println!("{}", idx);


    let idx = cluster_idx("c");

    println!("{}", idx);
}