#![allow(unused)]
use std::collections::HashMap;
use std::time::Duration;

use crate::functions::SceneValue;

#[derive(Debug)]
struct FaderValue {
    fixture_id: usize,
    channel: u16,
    from_value: u8,
    to_value: u8,
}

pub(crate) struct Fader {
    id: usize,
    to_id: usize,
    chaser_id: usize,
    values: Vec<FaderValue>,
    amount_duration: Duration,
    elapsed: Duration,
}

impl Fader {
    pub fn new(
        id: usize,
        to_id: usize,
        chaser_id: usize,
        prev_values: HashMap<usize, SceneValue>,
        curr_values: HashMap<usize, SceneValue>,
        duration: Duration,
    ) -> Self {
        // map (fixture_id, channel)->(from_value, to_value)
        let mut channel_map: HashMap<(usize, u16), (u8, u8)> = HashMap::new();

        for (fixture_id, values) in prev_values {
            for (ch, v) in values {
                //channel_map.insert((fixture_id, ch), (v, 0));
            }
        }
        for (fixture_id, values) in curr_values {
            for (ch, v) in values {
                /*channel_map
                .entry((fixture_id, ch))
                .and_modify(|values| values.1 = v)
                .or_insert((0, v));*/
            }
        }
        let values = channel_map
            .into_iter()
            .map(
                |((fixture_id, channel), (from_value, to_value))| FaderValue {
                    fixture_id,
                    channel,
                    from_value,
                    to_value,
                },
            )
            .collect();
        Self {
            id,
            to_id,
            chaser_id,
            values,
            amount_duration: duration,
            elapsed: Duration::ZERO,
        }
    }
}

/*impl FunctionRuntime for Fader {
    fn run(
        &mut self, //可変借用はselfのみ
        _function_infos: &std::collections::HashMap<usize, super::FunctionInfo>,
        fixtures: &std::collections::HashMap<usize, crate::fixture::Fixture>,
        tick_duration: Duration,
    ) -> Vec<FunctionCommand> {
        let mut commands = Vec::new();
        self.elapsed += tick_duration;
        if self.elapsed >= self.amount_duration {
            commands.push(FunctionCommand::StopFuntion(self.id()));

            commands.push(FunctionCommand::StartFunction(self.to_id));
            commands.push(FunctionCommand::StartFunction(self.chaser_id));
            return commands;
        }

        let ratio = self.elapsed.as_secs_f64() / self.amount_duration.as_secs_f64();
        let mut write_commands: Vec<FunctionCommand> = self
            .values
            .iter()
            .map(|v| {
                let start = v.from_value as f64;
                let end = v.to_value as f64;
                let new_value = (start + (end - start) * ratio).round() as u8;
                let address = DmxAddress::from_usize(
                    fixtures.get(&v.fixture_id).unwrap().address().as_usize() + v.channel as usize,
                )
                .unwrap();
                FunctionCommand::WriteUniverse {
                    fixture_id: 0,
                    channel: 0, //TODO: dummy implementation
                    value: new_value,
                }
            })
            .collect();
        commands.append(&mut write_commands);
        commands
    }
    fn function_type(&self) -> FunctionType {
        FunctionType::Fader
    }
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        ""
    }
}*/

#[cfg(test)]
mod tests {
    //use crate::{fixture::Fixture, functions::Context};

    //use super::*;

    #[test]
    fn test_fader_starts_and_stops() {
        /*let prev_values = vec![SceneValue1 {
            fixture_id: 0,
            channel: 5,
            value: 100,
        }];
        let curr_values = vec![SceneValue1 {
            fixture_id: 0,
            channel: 5,
            value: 250,
        }];
        let mut fader = Fader::new(
            2,
            1,
            0, //TODO: Chaserありで書き直しor疎結合化
            prev_values,
            curr_values,
            Duration::from_millis(500),
        );
        let mut dummy_fixtures = HashMap::new();
        dummy_fixtures.insert(0, Fixture::new(0, "", 10));
        let context = &Context {
            tick_duration: Duration::from_millis(100),
        };

        //1回目-4回目
        for i in 0..4 {
            let commands = fader.run(&HashMap::new(), &dummy_fixtures, context);

            assert_eq!(commands.len(), 1);
            assert!(commands[0].is_write_universe_and((15, 100 + 30 * (i + 1))));
        }

        //5回目
        let commands = fader.run(&HashMap::new(), &dummy_fixtures, context);
        let mut found_write = false;
        let mut found_start = false;
        let mut found_stop = false;
        commands.iter().for_each(|cmd| {
            if cmd.is_write_universe_and((15, 250)) {
                found_write = true
            } else if cmd.is_start_function_and(1) {
                found_start = true
            } else if cmd.is_stop_function_and(2) {
                found_stop = true
            }
        });
        //assert!(found_write);
        assert!(found_start);
        assert!(found_stop);*/
    }
}
