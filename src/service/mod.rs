pub mod api;
pub mod mqtt;
pub mod serial;

use api::ApiService;
use mqtt::MqttService;
use serial::SerialService;

use crate::AsyncMsg;
use dyno_core::crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub struct ServiceControl {
    pub serial: Option<SerialService>,
    pub api: Option<ApiService>,
    pub mqtt: Option<MqttService>,

    pub rx: Receiver<AsyncMsg>,
    pub tx: Sender<AsyncMsg>,
}

impl Default for ServiceControl {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceControl {
    #[inline]
    pub fn msg(&self) -> AsyncMsg {
        self.rx.try_recv().unwrap_or(AsyncMsg::Noop)
    }
}

impl ServiceControl {
    pub fn new() -> Self {
        let (tx, rx) = dyno_core::crossbeam_channel::unbounded();
        let serial = SerialService::new();
        let api = ApiService::new();
        let mqtt = MqttService::new();

        Self {
            serial,
            api,
            mqtt,
            rx,
            tx,
        }
    }
    pub fn init(&mut self, config: &dyno_core::DynoConfig) {
        match &self.api {
            Some(api) => api.set_active(config.clone(), self.tx.clone()),
            None => self.reconnect_api(config),
        }
        if self.serial.is_none() {
            self.reconnect_serial();
        }
    }
    pub fn deinit(&self) {
        if let Some(api) = &self.api {
            api.logout(self.tx.clone());
            api.set_non_active(self.tx.clone());
        }
        if let Some(mqtt) = &self.mqtt {
            mqtt.stop_subscribe();
        }
        if let Some(serial) = &self.serial {
            serial.stop();
        }
    }

    #[inline]
    pub fn api(&self) -> Option<&ApiService> {
        self.api.as_ref()
    }
    #[inline]
    pub fn serial(&self) -> Option<&SerialService> {
        self.serial.as_ref()
    }
    #[inline]
    pub fn mqtt(&self) -> Option<&MqttService> {
        self.mqtt.as_ref()
    }
    pub fn rx(&self) -> Receiver<AsyncMsg> {
        self.rx.clone()
    }
    pub fn tx(&self) -> Sender<AsyncMsg> {
        self.tx.clone()
    }

    #[inline]
    pub fn reconnect_api(&mut self, config: &dyno_core::DynoConfig) {
        if let Some(api) = ApiService::new() {
            crate::toast_success!("SUCCES! connected to Api Endpoint: {}", api.url);
            api.set_active(config.clone(), self.tx.clone());
            self.api = Some(api);
        }
    }

    #[inline]
    pub fn reconnect_serial(&mut self) {
        if let Some(serial) = SerialService::new() {
            crate::toast_success!(
                "SUCCES! connected to Serial: [{}]:[{}-{}]",
                serial.info.port_name,
                serial.info.vid,
                serial.info.pid
            );
            self.serial = Some(serial);
        }
    }

    #[inline]
    pub fn reconnect_mqtt(&mut self) {
        if let Some(mqtt) = MqttService::new() {
            crate::toast_success!("SUCCES! connected to MQTT Broker",);
            self.mqtt = Some(mqtt);
        }
    }
}
