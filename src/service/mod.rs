pub mod api;
pub mod serial;
pub mod ws;

use api::ApiService;
use serial::SerialService;

use crate::{paths::DynoPaths, toast_error, AsyncMsg};
use dyno_core::{
    chrono::{NaiveDateTime, Utc},
    crossbeam_channel::{Receiver, Sender},
};

use self::ws::WebSocketService;

#[derive(Clone)]
pub struct ServiceControl {
    data: [dyno_core::Data; 15],
    pub serial: Option<SerialService>,
    pub api: Option<ApiService>,
    pub websocket: WebSocketService,

    pub serial_time_start: Option<NaiveDateTime>,
    pub serial_time_stop: Option<NaiveDateTime>,

    pub rx: Receiver<AsyncMsg>,
    pub tx: Sender<AsyncMsg>,

    roll: usize,
    pub stream_data: bool,
}

impl ServiceControl {
    #[inline]
    pub fn msg(&self) -> AsyncMsg {
        self.rx.try_recv().unwrap_or(AsyncMsg::Noop)
    }
}

impl ServiceControl {
    pub fn new(paths: &DynoPaths) -> Self {
        let (tx, rx) = dyno_core::crossbeam_channel::unbounded();

        let serial = match SerialService::new(tx.clone()) {
            Ok(ok) => Some(ok),
            Err(err) => {
                toast_error!("{err}");
                None
            }
        };

        let api = ApiService::new(tx.clone());
        let websocket = WebSocketService::new(tx.clone(), paths);

        Self {
            data: Default::default(),
            serial,
            api,
            websocket,
            serial_time_start: None,
            serial_time_stop: None,
            rx,
            tx,
            roll: 0,
            stream_data: false,
        }
    }
    pub fn init(&mut self, config: &dyno_core::DynoConfig) {
        match &self.api {
            Some(api) => api.set_active(config.clone()),
            None => self.reconnect_api(config),
        }
        if self.serial.is_none() {
            self.reconnect_serial();
        }
    }
    pub fn deinit(&self) {
        if let Some(api) = &self.api {
            api.logout();
            api.set_non_active();
        }
        self.websocket.stop();

        if let Some(serial) = &self.serial {
            serial.stop();
        }
    }
    pub fn rx(&self) -> Receiver<AsyncMsg> {
        self.rx.clone()
    }

    #[inline]
    pub fn api(&self) -> Option<&ApiService> {
        self.api.as_ref()
    }
    #[inline]
    pub fn reconnect_api(&mut self, config: &dyno_core::DynoConfig) {
        if let Some(api) = ApiService::new(self.tx.clone()) {
            crate::toast_success!("SUCCES! connected to Api Endpoint: {}", api.url);
            api.set_active(config.clone());
            self.api = Some(api);
        }
    }

    pub fn start_stream(&mut self) {
        self.websocket.start();
        self.stream_data = true;
    }
    pub fn stop_stream(&mut self) {
        self.websocket.stop();
        self.stream_data = false;
    }

    pub fn send_stream_data(&mut self, data: &dyno_core::Data) {
        if !self.stream_data {
            return;
        };
        if self.on_data(data) {
            if let Err(err) = self.websocket.send_data(&self.data) {
                dyno_core::log::error!("{err}");
            }
        }
    }

    pub fn on_data(&mut self, data: &dyno_core::Data) -> bool {
        self.data[self.roll] = *data;
        self.roll += 1;
        if self.roll >= 15 {
            self.roll = 0;
            return true;
        }
        false
    }
}

impl ServiceControl {
    #[inline]
    pub fn ws(&self) -> &WebSocketService {
        &self.websocket
    }
    #[inline]
    pub fn mqtt_mut(&mut self) -> &mut WebSocketService {
        &mut self.websocket
    }

    #[inline]
    pub fn reconnect_mqtt(&mut self) {
        self.websocket.stop();
        self.websocket.start();
    }
}

#[allow(unused)]
impl ServiceControl {
    #[inline]
    pub fn serial(&self) -> Option<&SerialService> {
        self.serial.as_ref()
    }

    #[inline]
    pub fn is_serial_connected(&self) -> bool {
        self.serial.as_ref().is_some_and(|x| x.is_open())
    }

    #[inline]
    pub fn start_serial(&mut self) {
        if let Some(serial) = &self.serial {
            if let Err(err) = serial.start() {
                toast_error!("Serial Service Failed to start - {err}")
            }
            self.serial_time_start = Some(Utc::now().naive_utc());
        }
    }
    #[inline]
    pub fn stop_serial(&mut self) {
        if let Some(serial) = &self.serial {
            serial.stop();
            self.serial_time_stop = Some(Utc::now().naive_utc());
        }
    }

    #[inline]
    pub fn reconnect_serial(&mut self) {
        if let Ok(serial) = SerialService::new(self.tx.clone()) {
            crate::toast_success!(
                "SUCCES! connected to Serial: [{}]:[{}-{}]",
                serial.info.port_name,
                serial.info.vid,
                serial.info.pid
            );
            self.serial = Some(serial);
        }
    }
}
