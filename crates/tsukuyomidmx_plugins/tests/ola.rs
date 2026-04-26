use std::time::Duration;

use tsukuyomidmx_core::{
    plugins::{AsyncPlugin, DmxFrame},
    prelude::UniverseId,
};
use tsukuyomidmx_plugins::OlaPlugin;

const OLAD_PORT: &'static str = "9011";

#[tokio::test]
async fn ola_plugin_can_send_dmx() {
    /*tokio::process::Command::new("olad")
        .args(["--rpc-port", OLAD_PORT, "--no-http"])
        .kill_on_drop(true)
        .spawn()
        .unwrap();

    tokio::time::sleep(Duration::from_millis(1500)).await;*/

    let mut client = OlaPlugin::builder()
        .port(OLAD_PORT)
        .connect()
        .await
        .unwrap();

    let mut data = [0; 512];
    data[0] = 255;
    client
        .send_dmx(UniverseId::new(0), DmxFrame { data })
        .await
        .unwrap();
}
