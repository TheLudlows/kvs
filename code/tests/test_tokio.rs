
use tokio::runtime::Builder;
use tokio::task::LocalSet;

#[test]
fn main() {
    // build runtime
    let runtime = Builder::new_multi_thread()
        .worker_threads(8)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();
    // use runtime ...

    let p = Box::new(1);
}