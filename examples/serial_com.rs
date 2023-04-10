use dynotest_app::service;
use dynotest_app::types::{DynoErr, DynoResult};

fn main() -> DynoResult<()> {
    let port = service::get_dyno_port()?;
    let mut serial_service = service::SerialService::new();
    let handle = match port {
        Some(p) => serial_service.open(&p.port_name, 500000)?,
        None => return Err(DynoErr::IO("Serial Port Error Error".to_owned())),
    };

    while !handle.is_finished() {
        match serial_service.handle() {
            Ok(data) => println!("DATA: {data}"),
            Err(err) => eprintln!("ERROR: {err}"),
        }
    }
    handle.join().expect("Cannot Join Handle");
    Ok(())
}
