use std::fmt::Debug;
use tonic::transport::{Channel, Endpoint};
use tracing::info;
use tsukuyomidmx_core::plugins::{AsyncPlugin, OutputPluginId};

use crate::ola::ola_proto::{DmxData, ola_server_service_client::OlaServerServiceClient};

mod ola_proto {
    tonic::include_proto!("ola.proto");
    tonic::include_proto!("ola.rpc");
}

const OLAD_DEFAULT_PORT: &'static str = "https://127.0.0.1:9010";

#[derive(Debug)]
pub struct OlaPlugin {
    client: OlaServerServiceClient<Channel>,
    id: OutputPluginId,
}

pub struct Builder {
    port: String,
}

impl OlaPlugin {
    /// Create OlaPlugin with default configuration. To configure, use [`OlaPlugin::builder()`].
    pub async fn new() -> Result<Self, tonic::transport::Error> {
        let client = OlaServerServiceClient::connect(OLAD_DEFAULT_PORT).await?;
        info!(port = OLAD_DEFAULT_PORT, "successfully conneted to olad");
        Ok(Self {
            client,
            id: OutputPluginId::new(),
        })
    }

    pub fn builder() -> Builder {
        Builder {
            port: OLAD_DEFAULT_PORT.try_into().unwrap(),
        }
    }
}

impl Builder {
    /// Port to send RPCs (should same as `--rpc-port` option of olad).
    #[must_use]
    pub fn port(self, port: &str) -> Builder {
        Builder {
            port: format!("https://127.0.0.1:{}", port),
        }
    }

    pub async fn connect(self) -> Result<OlaPlugin, tonic::transport::Error> {
        let port = self.port.clone();
        let client = OlaServerServiceClient::connect(self.port).await?;
        info!(?port, "successfully connected to olad");

        Ok(OlaPlugin {
            client,
            id: OutputPluginId::new(),
        })
    }
}

impl AsyncPlugin for OlaPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    async fn send_dmx(
        &mut self,
        universe_id: tsukuyomidmx_core::prelude::UniverseId,
        dmx_data: tsukuyomidmx_core::plugins::DmxFrame,
    ) -> Result<(), std::io::Error> {
        let req = DmxData {
            universe: universe_id.as_usize().try_into().unwrap(),
            data: dmx_data.as_slice().iter().cloned().collect(), // OPTIM: ここのcloneはどうにかならないものか
            priority: None,
        };
        self.client
            .stream_dmx_data(req)
            .await
            .map_err(|status| std::io::Error::new(std::io::ErrorKind::Other, status))?;
        Ok(())
    }
}
