#![allow(unused)]
pub struct Collection {
    id: usize,
    name: String,
    functions: Vec<usize>,
}

impl Collection {
    pub fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: String::from(name),
            functions: Vec::new(),
        }
    }

    // TODO: Engineに登録されていないやつを防ぎたい。
    // Collection内でやろうとすると所有権地獄になると思うので、
    // Engine::push_functionが呼ばれたときに内部をチェックするのが現実的か。
    pub fn push_function(&mut self, id: usize) {
        self.functions.push(id);
    }
}

/*impl Function for Collection {
    fn run(
        &mut self, //可変借用はselfのみ
        _function_infos: &HashMap<usize, FunctionInfo>,
        _fixtures: &HashMap<usize, Fixture>,
        _tick_duration: Duration,
    ) -> Vec<FunctionCommand> {
        //TODO: コレクションの終了時中身をストップさせるのはだれ？
        self.functions
            .iter()
            .map(|id| FunctionCommand::StartFunction(*id))
            .collect()
    }

    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn function_type(&self) -> FunctionType {
        FunctionType::Collection
    }
}*/

#[cfg(test)]
mod tests {
    /*use std::time::Duration;

    use crate::functions::Scene;

    use super::*;*/

    #[test]
    fn test_collection_starts_functions() {
        /*let mut collection = Collection::new(0, "collection");

        let mut scene1 = Scene::new(1, "scene");
        scene1.push_value(SceneValue1 {
            fixture_id: 0,
            channel: 0,
            value: 255,
        });
        collection.push_function(scene1.id());

        let mut scene2 = Scene::new(2, "scene2");
        scene2.push_value(SceneValue1 {
            fixture_id: 0,
            channel: 1,
            value: 100,
        });
        collection.push_function(scene2.id());

        let commands = collection.run(
            &HashMap::new(),
            &HashMap::new(),
            &Context {
                tick_duration: Duration::ZERO,
            },
        );

        let mut found_start_1 = false;
        let mut found_start_2 = false;
        commands.iter().for_each(|cmd| {
            if cmd.is_start_function_and(1) {
                found_start_1 = true
            } else if cmd.is_start_function_and(2) {
                found_start_2 = true
            } else if cmd.is_stop_function() {
                panic!("unexpected stop")
            } else if cmd.is_write_universe() {
                panic!("unexpected write")
            }
        });
        assert!(found_start_1 && found_start_2)*/
    }
}
