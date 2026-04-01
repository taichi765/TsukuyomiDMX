use std::sync::mpsc::Sender;

use crate::{AppWindow, FadersStore};
use slint::{ComponentHandle, SharedString, ToSharedString};
use tsukuyomi_core::{engine::EngineCommand, fixture::FixtureId};
use uuid::Uuid;

pub fn setup_fader_view(ui: &AppWindow, command_tx: Sender<EngineCommand>, fixture_id: FixtureId) {
    ui.global::<FadersStore>()
        .set_selected_fixture(fixture_id.to_shared_string());
    ui.global::<FadersStore>().on_value_changed(
        move |fixture_id: SharedString, channel: SharedString, value: i32| {
            command_tx
                .send(EngineCommand::SetLiveValue {
                    fixture_id: FixtureId::from(Uuid::parse_str(&fixture_id).unwrap()),
                    channel: channel.to_string(),
                    value: value as u8,
                })
                .expect("failed to send command to engine");
        },
    );
}
