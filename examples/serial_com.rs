use dyno_core::crossbeam_channel::unbounded;
use dyno_core::{tokio, LoggerBuilder};
use dynotest_app::{service, AsyncMsg};

fn main() {
    LoggerBuilder::new()
        .set_max_level(dyno_core::log::LevelFilter::Debug)
        .build_console_logger()
        .unwrap();

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

    let Some(serial_service) = service::serial::SerialService::new() else { return };
    let (tx, rx) = unbounded();
    let handle = match serial_service.start(tx) {
        Ok(k) => k,
        Err(err) => {
            eprintln!("ERROR: Failed to start serial service - {err}");
            return;
        }
    };

    let mut data_received = 0usize;
    while !handle.is_finished() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        match rx.try_recv() {
            Ok(AsyncMsg::OnSerialData(serial_data)) => {
                data_received += 1;
                eprintln!("[{data_received}] - DATA: {serial_data}")
            }
            Ok(AsyncMsg::OnError(err)) => eprintln!("[{err}]"),
            _ => (),
        }
    }
    eprintln!("Finish!");
}
