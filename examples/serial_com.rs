use dyno_core::tokio;
use dynotest_app::{init_logger, service};

fn main() {
    init_logger("");

    let rt = tokio::runtime::Runtime::new().expect("Unable to create tokio's Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        })
    });

    let mut serial_service = match service::SerialService::new() {
        Ok(k) => k,
        Err(err) => {
            eprintln!("ERROR: Failed to crate service - {err}");
            return;
        }
    };
    let handle = match serial_service.start() {
        Ok(k) => k,
        Err(err) => {
            eprintln!("ERROR: Failed to start serial service - {err}");
            return;
        }
    };

    let mut data_received = 0usize;
    while !handle.is_finished() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        match serial_service.handle() {
            Some(data) => {
                data_received += 1;
                eprintln!("[{data_received}] - DATA: {data}")
            }
            None => continue,
        }
    }
    eprintln!("Finish!");
}
