use std::collections::HashMap;
use std::time::Duration;

use super::{FunctionData, FunctionRuntime};
use crate::{
    fixture::FixtureId,
    functions::{FunctionCommand, FunctionDataGetters, FunctionId},
};

pub type SceneValue = HashMap<String, u8>;

#[derive(Debug)]
pub struct StaticSceneData {
    id: FunctionId,
    name: String,
    /// fixture_id->values
    values: HashMap<FixtureId, SceneValue>,
}

impl FunctionDataGetters for StaticSceneData {
    fn id(&self) -> FunctionId {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl StaticSceneData {
    pub fn new(name: &str) -> Self {
        Self {
            id: FunctionId::new(),
            name: String::from(name),
            values: HashMap::new(),
        }
    }

    pub fn values(&self) -> &HashMap<FixtureId, SceneValue> {
        &self.values
    }

    pub fn insert_value(&mut self, fixture_id: FixtureId, value: SceneValue) {
        self.values.insert(fixture_id, value);
    }
}

pub struct StaticSceneRuntime {}

impl StaticSceneRuntime {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl FunctionRuntime for StaticSceneRuntime {
    fn run(&mut self, data: &FunctionData, _tick_duration: Duration) -> Vec<FunctionCommand> {
        let FunctionData::StaticScene(data) = data else {
            panic!("unknown data type")
        };

        let mut commands = Vec::new();
        for (fixture_id, scene_value) in &data.values {
            for (channel, value) in scene_value {
                commands.push(FunctionCommand::WriteUniverse {
                    fixture_id: *fixture_id,
                    channel: channel.clone(),
                    value: *value,
                });
            }
        }
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn static_scene_runtime_writes_commands_for_single_fixture() {
        let mut scene = StaticSceneData::new("test");
        let mut sv = SceneValue::new();
        sv.insert("Intensity".into(), 128);
        sv.insert("Red".into(), 255);
        let fixture_id = FixtureId::from(Uuid::new_v4());
        scene.insert_value(fixture_id, sv);

        let mut runtime = StaticSceneRuntime::new();
        let commands = runtime.run(
            &FunctionData::StaticScene(scene),
            Duration::from_millis(100),
        );

        let mut found = vec![];
        for cmd in commands {
            match cmd {
                FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => {
                    found.push((fixture_id, channel, value));
                }
                _ => panic!("unexpected command"),
            }
        }
        assert_eq!(found.len(), 2);
        assert!(found.contains(&(fixture_id, "Intensity".into(), 128)));
        assert!(found.contains(&(fixture_id, "Red".into(), 255)));
    }

    #[test]
    fn static_scene_runtime_writes_commands_for_multiple_fixtures() {
        let mut scene = StaticSceneData::new("multi");
        let fixture_id_1 = FixtureId::from(Uuid::new_v4());
        let fixture_id_2 = FixtureId::from(Uuid::new_v4());

        let mut sv1 = SceneValue::new();
        sv1.insert("Intensity".into(), 10);

        scene.insert_value(fixture_id_1, sv1);

        let mut sv2 = SceneValue::new();
        sv2.insert("Red".into(), 200);
        sv2.insert("Blue".into(), 201);
        scene.insert_value(fixture_id_2, sv2);

        let mut runtime = StaticSceneRuntime::new();
        let commands = runtime.run(&FunctionData::StaticScene(scene), Duration::from_millis(50));

        let expected: HashSet<_> = vec![
            (fixture_id_1, "Intensity".into(), 10),
            (fixture_id_2, "Red".into(), 200),
            (fixture_id_2, "Blue".into(), 201),
        ]
        .into_iter()
        .collect();

        let found: HashSet<_> = commands
            .into_iter()
            .map(|cmd| match cmd {
                FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => (fixture_id, channel, value),
                _ => panic!("unexpected command"),
            })
            .collect();

        assert_eq!(found, expected);
    }
}
