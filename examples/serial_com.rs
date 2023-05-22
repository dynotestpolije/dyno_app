use dynotest_app::service;

fn main() {
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

    while !handle.is_finished() {
        match serial_service.handle() {
            Some(data) => println!("DATA: {data}"),
            None => eprintln!("Data not handled"),
        }
    }
    handle.join().expect("Cannot Join Handle");
}
