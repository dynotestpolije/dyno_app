#![allow(unused)]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use dyno_core::{
    crossbeam_channel::Sender,
    ignore_err,
    lazy_static::lazy_static,
    uuid::{fmt::Simple, Uuid},
    BinSerializeDeserialize, DynoConfig, DynoErr, DynoResult,
};
use eframe::epaint::mutex::Mutex;
use futures::StreamExt;
use mqtt::{ConnectOptions, CreateOptions};
use paho_mqtt as mqtt;

use crate::{toast_error, AsyncMsg};

#[derive(Clone)]
pub struct MqttService {
    uuid: Arc<Mutex<Option<Uuid>>>,
    client: mqtt::AsyncClient,

    sub: Arc<AtomicBool>,
    start: Arc<AtomicBool>,
    data: [dyno_core::Data; 20],
    roll: usize,
}

impl MqttService {
    pub fn new() -> Option<Self> {
        let broker = std::env::var("DYNO_MQTT_BROKER").unwrap_or_else(|err| {
            toast_error!(
                "Failed to Get DYNO_MQTT_BROKER from EnvVar, defaulting to [broker.hivemq.com:1883] - {err}"
            );
            "ws://broker.hivemq.com:1883".to_owned()
        });

        let client = match mqtt::CreateOptionsBuilder::new_v3()
            .server_uri(broker)
            .client_id(concat!(
                env!("CARGO_PKG_NAME"),
                "_",
                env!("CARGO_PKG_VERSION")
            ))
            .create_client()
        {
            Ok(ok) => Some(ok),
            Err(err) => {
                toast_error!("Failed to create MQTT Client - {err}");
                None
            }
        }?;

        Some(Self {
            client,
            uuid: Default::default(),
            data: Default::default(),
            sub: Arc::new(AtomicBool::new(false)),
            start: Arc::new(AtomicBool::new(false)),
            roll: 0,
        })
    }

    pub fn on_data(&mut self, data: &dyno_core::Data) -> bool {
        if (self.roll + 1) >= self.data.len() {
            self.roll = 0;
            return true;
        }
        self.data[self.roll] = *data;
        self.roll += 1;
        false
    }
}

use dyno_core::paste;
macro_rules! impl_cmd {
    ($name:ident {$($cmd:ident),*}) => {
        #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
        enum $name {
            $($cmd),*,
            #[default]
            Noop,
        }
        paste::paste!{
            impl $name {
                const TOPICS: &[&'static str] = &[
                    $(concat!("E32201406/dyno/", stringify!([<$cmd:snake>]))),*
                ];
                const QOS: [i32; Self::TOPICS.len()] = [
                    $(
                        #[doc = "Qos for `" $cmd "` topic."]
                        mqtt::QOS_2
                    ),*
                ];
                pub fn from_optional_string(string: Option<&'_ str>) -> Self {
                    match string {
                        $(Some(stringify!([<$cmd:snake>])) => Self::$cmd,)*
                        Some(_) => Self::Noop,
                        _ => Self::Noop,
                    }
                }
            }
        }
    };
}
impl_cmd!(Command { Start, Stop, Will });

impl MqttService {
    pub fn start_subscribe(&self, data: DynoConfig, tx: Sender<AsyncMsg>) {
        self.sub.store(true, Ordering::Relaxed);
        let sub = self.sub.clone();

        let mut client = self.client.clone();
        let uuid = self.uuid.clone();
        let start = self.start.clone();

        let future_fn = async move {
            // Get message stream before connecting.
            let mut strm = client.get_stream(25);
            // Define the set of options for the connection
            let lwt = mqtt::Message::new("will", "desktop", mqtt::QOS_2);
            // Create the connect options, explicitly requesting MQTT v3.x
            let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
                .keep_alive_interval(std::time::Duration::from_secs(120))
                .clean_session(true)
                .will_message(lwt)
                .finalize();

            if let Err(err) = client.connect(conn_opts).await {
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::service_error(err))));
                return;
            }

            if let Err(err) = client.subscribe_many(Command::TOPICS, &Command::QOS).await {
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::service_error(err))));
                return;
            }
            while let Some(msg_opt) = strm.next().await {
                if !sub.load(Ordering::Relaxed) {
                    break;
                }
                let Some(msg) = msg_opt else {
                    continue;
                };

                match Command::from_optional_string(msg.topic().strip_prefix("E32201406/dyno/")) {
                    Command::Start => {
                        let payload = match dyno_core::uuid::Uuid::from_slice(msg.payload()) {
                            Ok(ok) => ok,
                            Err(err) => {
                                ignore_err!(tx.send(AsyncMsg::error(DynoErr::service_error(err))));
                                continue;
                            }
                        };
                        {
                            let mut locked = uuid.lock();
                            *locked = Some(payload);
                        }

                        if let Err(err) =
                            Self::publish(client.clone(), format!("{payload}/info"), data.clone())
                                .await
                        {
                            ignore_err!(tx.send(AsyncMsg::error(err)));
                            continue;
                        }
                        start.store(true, Ordering::Relaxed);
                    }
                    Command::Stop => start.store(true, Ordering::Relaxed),
                    Command::Will => {}
                    Command::Noop => {}
                }
            }
        };
        dyno_core::tokio::spawn(future_fn);
    }

    #[inline]
    pub fn stop_subscribe(&self) {
        self.sub.store(false, Ordering::Relaxed);
    }

    #[inline]
    pub fn is_subscribe_running(&self) -> bool {
        self.sub.load(Ordering::Relaxed)
    }
}

impl MqttService {
    pub fn is_started(&self) -> bool {
        self.start.load(Ordering::Relaxed)
    }

    pub async fn publish(
        client: mqtt::AsyncClient,
        topic: String,
        payload: impl BinSerializeDeserialize,
    ) -> DynoResult<()> {
        let msg = mqtt::Message::new(topic, payload.serialize_bin()?, mqtt::QOS_2);
        client.publish(msg).await.map_err(DynoErr::service_error)?;
        Ok(())
    }

    fn pre_configure(&self) -> DynoResult<(Simple, mqtt::AsyncClient)> {
        if !self.client.is_connected() {
            return Err(DynoErr::service_error("MQTT Client is not connected!"));
        }
        let uuid = self
            .uuid
            .lock()
            .ok_or(DynoErr::service_error("No Uuid set from destination"))?
            .simple();
        let client = self.client.clone();

        Ok((uuid, client))
    }

    pub fn publish_data(&self, tx: Sender<AsyncMsg>) -> DynoResult<()> {
        let (uuid, client) = self.pre_configure()?;
        let data = self.data;

        let future_block = async move {
            if let Err(err) = Self::publish(client, format!("{uuid}/data"), data).await {
                ignore_err!(tx.send(AsyncMsg::error(err)))
            }
        };
        dyno_core::tokio::spawn(future_block);
        Ok(())
    }

    pub fn publish_info(&self, info: DynoConfig, tx: Sender<AsyncMsg>) -> DynoResult<()> {
        let (uuid, client) = self.pre_configure()?;
        let future_block = async move {
            if let Err(err) = Self::publish(client, format!("{uuid}/info"), info).await {
                ignore_err!(tx.send(AsyncMsg::error(err)))
            }
        };
        dyno_core::tokio::spawn(future_block);
        Ok(())
    }
}
