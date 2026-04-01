#![allow(unused)]
use uuid::Uuid;

use super::FunctionData;
use crate::functions::{FunctionId, FunctionRuntime};
use crate::{functions::FunctionCommand, functions::FunctionDataGetters};
use std::{collections::HashMap, time::Duration};

//TODO: フェードインの実装

#[derive(Debug)]
pub struct ChaserData {
    id: FunctionId,
    name: String,
    ///step_number->step
    steps: HashMap<usize, ChaserStep>,
}

impl FunctionDataGetters for ChaserData {
    fn id(&self) -> FunctionId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ChaserData {
    pub fn new(name: &str) -> Self {
        Self {
            id: FunctionId::new(),
            name: String::from(name),
            steps: HashMap::new(),
        }
    }
    pub fn add_step(&mut self, function_id: FunctionId, hold: Duration, fade_in: Duration) {
        self.steps.insert(
            self.steps.len(),
            ChaserStep {
                function_id,
                fade_in,
                hold,
            },
        );
    }
}

pub struct ChaserRuntime {
    time_in_current_step: Duration,
    current_step_num: usize,
}

impl FunctionRuntime for ChaserRuntime {
    fn run(&mut self, data: &FunctionData, tick_duration: Duration) -> Vec<FunctionCommand> {
        let FunctionData::Chaser(data) = data else {
            panic!("unknown function data")
        };

        let mut commands = Vec::new();
        self.time_in_current_step += tick_duration; //時間を進める

        //if self.time_in_current_step < self.current_step().duration() {
        commands.push(FunctionCommand::StartFunction(
            FunctionId::from(Uuid::nil()), //TODO: self.current_step().function_id,
        )); //べき等
        return commands;
        //}

        //ステップ移行
        /*let function_info = function_infos
            .get(&self.current_step().function_id)
            .unwrap();
        match function_info.function_type {
            FunctionType::Scene => {
                commands.push(FunctionCommand::StopFuntion(
                    self.current_step().function_id,
                ));
                self.current_step_num += 1;
                self.time_in_current_step = Duration::ZERO;

                //最後のステップまで行った
                if data.steps.len() == self.current_step_num {
                    commands.push(FunctionCommand::StopFuntion(data.id));
                    return commands;
                }

                //次のステップをstart
                if self.current_step().fade_in.is_zero() {
                    //フェードインが必要ない場合
                    commands.push(FunctionCommand::StartFunction(
                        self.current_step().function_id,
                    ));
                } else {
                    commands.push(FunctionCommand::StopFuntion(self.id));
                    commands.push(FunctionCommand::StartFade {
                        from_id: self
                            .steps
                            .get(&(self.current_step_num - 1))
                            .unwrap()
                            .function_id,
                        to_id: self.current_step().function_id,
                        chaser_id: self.id,
                        duration: self.current_step().fade_in,
                    });
                }
            }
            _ => unimplemented!(),
        }*/
        commands
    }
}

impl ChaserRuntime {
    pub(crate) fn new() -> Self {
        Self {
            time_in_current_step: Duration::ZERO,
            current_step_num: 0,
        }
    }

    /*fn current_step(&self) -> &ChaserStep {
        self.steps
            .get(&self.current_step_num)
            .expect(format!("step num {} not found", self.current_step_num).as_str())
    }*/
}

#[derive(Debug)]
struct ChaserStep {
    function_id: FunctionId,
    fade_in: Duration,
    hold: Duration,
}

impl ChaserStep {
    fn duration(&self) -> Duration {
        self.hold + self.fade_in
    }
}

#[cfg(test)]
mod tests {

    /*use super::*;

    #[test]
    fn test_chaser_advances_step_after_hold_time() {
        let mut chaser = ChaserData::new(0, "Test Chaser");
        chaser.add_step(1, Duration::from_millis(500), Duration::ZERO);
        chaser.add_step(2, Duration::from_millis(1000), Duration::ZERO);

        let mut dummy_infos = HashMap::new();
        /*dummy_infos.insert(
            1,
            FunctionInfo {
                id: 1,
                function_type: FunctionType::Scene,
            },
        );
        dummy_infos.insert(
            2,
            FunctionInfo {
                id: 2,
                function_type: FunctionType::Scene,
            },
        );*/

        let tick_duration = Duration::from_millis(120);

        //1回目: スタートしてるか
        let commands = chaser.run(&dummy_infos, &HashMap::new(), tick_duration);
        commands
            .iter()
            .find(|cmd| cmd.is_start_function_and(1))
            .unwrap();

        //2回目~4回目: ストップしてないか
        for _ in 0..3 {
            let commands = chaser.run(&dummy_infos, &HashMap::new(), tick_duration);

            assert_eq!(chaser.current_step_num, 0);
            commands.iter().for_each(|cmd| {
                if cmd.is_stop_function() {
                    panic!("unexpected stop");
                }
                if cmd.is_start_function_and(2) {
                    panic!("unexpected start")
                }
            });
        }

        // ステップが進む
        let commands = chaser.run(&dummy_infos, &HashMap::new(), tick_duration);
        assert_eq!(chaser.current_step_num, 1);
        let mut found_start = false;
        let mut found_stop = false;
        for cmd in commands {
            if cmd.is_start_function_and(2) {
                found_start = true
            }
            if cmd.is_stop_function_and(1) {
                found_stop = true
            }
        }
        assert!(found_start && found_stop);
    }
    #[test]
    fn test_chaser_starts_fade() {}*/
}
