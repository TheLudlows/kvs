
use tokio::runtime::Builder;
#[test]
fn main() {
    // build runtime
    let runtime = Builder::new_multi_thread()
        .core_threads(4)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();

    // use runtime ...
}