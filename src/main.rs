mod runtime;
mod future;

fn main() {
    let (executor, spawner) = runtime::new_executor_and_spawner();

    spawner.spawn(async {
        println!("sleep for 1 second!");
        future::sleep(std::time::Duration::from_secs(1)).await;
        println!("wake up!");
    });

    std::mem::drop(spawner);  // dropping tx half

    executor.run();
}