#![allow(unused_variables)]

mod parallel;
pub use parallel::{ParallelEffectBody, ParallelEffectSpecBody, ParallelEffectTemplateBody};
mod sequence;
pub use sequence::{SequenceEffectBody, SequenceEffectSpecBody, SequenceEffectTemplateBody};
mod simple;
pub use simple::{SimpleEffectBody, SimpleEffectSpecBody, SimpleEffectTemplateBody};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::doc::DocStateView;
use crate::fixture::{FixtureId, FixtureTag};
use std::collections::HashMap;
use std::time::Duration;

declare_id_newtype!(EffectSpecId);
declare_id_newtype!(EffectTemplateId);
declare_id_newtype!(EffectId);

pub trait EffectRegistry<Spec, Template> {
    fn with_spec<F, R>(&self, spec_id: EffectSpecId, f: F) -> R
    where
        F: FnOnce(&Spec) -> R;

    fn with_template<F, R>(&self, tmpl_id: EffectTemplateId, f: F) -> R
    where
        F: FnOnce(&Template) -> R;
}

/// [`FunctionRuntime::run()`] returns this and [`Engine`][crate::engine::Engine] evaluates the command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectCommand {
    /// if the function is already started, `Engine` do nothing.
    StartEffect(EffectId),
    /// 実行中のFunctionをstopする
    StopEffect,
    WriteUniverse {
        fixture_id: FixtureId,
        channel: usize,
        value: u8,
    },
    /*StartFade {
        from_id: EffectId,
        to_id: EffectId,
        chaser_id: EffectId,
        duration: Duration,
    },*/
}

/// self-contained runtime.
pub(crate) trait EffectRuntime: Send {
    fn run(&mut self, elapsed: Duration) -> Vec<EffectCommand>;

    fn first_frame_hint(&self) -> Vec<EffectCommand>;

    fn last_frame_hint(&self) -> Vec<EffectCommand>;
}

/// bind_to()でFixtureに関連付けられる前のfunction.
///
/// Dimmer, Colorなどmodel-agnosticなチャンネルを制御する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct EffectSpec {
    #[getter(copy)]
    id: EffectSpecId,
    name: String,
    body: EffectSpecBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSpecBody {
    Simple(SimpleEffectSpecBody),
    Sequence(SequenceEffectSpecBody),
    Parallel(ParallelEffectSpecBody),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectSpecChange {
    Rename(String),
    Simple(SimpleEffectSpecBody),
    Sequence(SequenceEffectSpecBody),
    Parallel(ParallelEffectSpecBody),
}

impl EffectSpec {
    pub fn new_simple(name: impl Into<String>) -> Self {
        Self {
            id: EffectSpecId::new(),
            name: name.into(),
            body: EffectSpecBody::Simple(SimpleEffectSpecBody::new()),
        }
    }

    pub(crate) fn apply_change(&mut self, change: EffectSpecChange) {
        match change {
            EffectSpecChange::Rename(name) => {
                self.name = name;
            }
            EffectSpecChange::Simple(body) => {
                self.body = EffectSpecBody::Simple(body);
            }
            EffectSpecChange::Sequence(body) => self.body = EffectSpecBody::Sequence(body),
            EffectSpecChange::Parallel(body) => self.body = EffectSpecBody::Parallel(body),
        }
    }
}

impl EffectSpecBody {
    fn resolve_props(
        &self,
        fixtures: &FixtureQuery,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::Simple(body) => body.resolve_props(fixtures, given_props, doc),
            Self::Sequence(body) => body.resolve_props(given_props, doc),
            Self::Parallel(body) => body.resolve_props(given_props, doc),
        }
    }
}

/// Propsに代入することで[`Effect`]を得られる。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct EffectTemplate {
    #[getter(copy)]
    id: EffectTemplateId,
    name: String,
    body: EffectTemplateBody,
}

/// [`EffectTemplate`]のbody。
///
/// `Sequence`のstep等として埋め込める。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectTemplateBody {
    Simple(SimpleEffectTemplateBody),
    Sequence(SequenceEffectTemplateBody),
    Parallel(ParallelEffectTemplateBody),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectTemplateChange {
    Rename(String),
    Simple(SimpleEffectTemplateBody),
    Sequence(SequenceEffectTemplateBody),
    Parallel(ParallelEffectTemplateBody),
}

impl EffectTemplate {
    pub fn new_simple(name: impl Into<String>) -> Self {
        Self {
            id: EffectTemplateId::new(),
            name: name.into(),
            body: EffectTemplateBody::Simple(SimpleEffectTemplateBody::new()),
        }
    }

    // TODO: 個別のchangeを用意したほうがいいか？
    pub(crate) fn apply_change(&mut self, change: EffectTemplateChange) {
        match change {
            EffectTemplateChange::Rename(name) => {
                self.name = name;
            }
            EffectTemplateChange::Simple(body) => {
                self.body = EffectTemplateBody::Simple(body);
            }
            EffectTemplateChange::Sequence(body) => self.body = EffectTemplateBody::Sequence(body),
            EffectTemplateChange::Parallel(body) => self.body = EffectTemplateBody::Parallel(body),
        }
    }
}

/// bind_to()でFixtureに関連付けたあとのfunction.
///
/// Goboなどmodel-specificなチャンネルを制御する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct Effect {
    #[getter(copy)]
    id: EffectId,
    name: String,
    body: EffectBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectBody {
    Simple(SimpleEffectBody),
    Sequence(SequenceEffectBody),
    Parallel(ParallelEffectBody),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectChange {
    Rename(String),
    Simple(SimpleEffectBody),
    Sequence(SequenceEffectBody),
    Parallel(ParallelEffectBody),
}

impl Effect {
    pub fn new_simple(name: impl Into<String>) -> Effect {
        Effect {
            id: EffectId::new(),
            name: name.into(),
            body: EffectBody::Simple(SimpleEffectBody::new()),
        }
    }

    pub fn new_sequence(name: impl Into<String>) -> Self {
        todo!()
        /*Self {
            id: EffectId::new(),
            name: name.into(),
            body: EffectBody::Sequence(SequenceEffectBody::),
        }*/
    }

    pub fn new_parallel(name: impl Into<String>) -> Self {
        Self {
            id: EffectId::new(),
            name: name.into(),
            body: EffectBody::Parallel(ParallelEffectBody::new()),
        }
    }

    pub(crate) fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        doc.create_runtime(self.id)
    }

    pub(crate) fn apply_change(&mut self, change: EffectChange) {
        match change {
            EffectChange::Rename(name) => {
                self.name = name;
            }
            EffectChange::Simple(body) => {
                self.body = EffectBody::Simple(body);
            }
            EffectChange::Sequence(body) => self.body = EffectBody::Sequence(body),
            EffectChange::Parallel(body) => self.body = EffectBody::Parallel(body),
        }
    }
}

pub struct Diagnostics {
    inner: Vec<DiagnosticItem>,
}

struct DiagnosticItem {
    message: String,
}

impl Diagnostics {
    pub fn push_err(&mut self, message: impl Into<String>) {
        self.inner.push(DiagnosticItem {
            message: message.into(),
        });
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expression {
    Value(Value),
    Prop(String),
}

/// Propsとして取れる型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Duration,
    Dimmer,
    Color,
    FixtureQuery,
}

impl Type {
    /// この型のdefaultのvalueを返す
    pub fn default_value(&self) -> Value {
        match self {
            Self::Duration => Value::Duration(Duration::default()),
            Self::Dimmer => Value::Dimmer(0),
            Self::Color => Value::Color([0, 0, 0]),
            Self::FixtureQuery => Value::FixtureQuery(FixtureQuery::default()),
        }
    }
}

/// Propsとして渡せる値
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Duration(Duration),
    Dimmer(u8),
    Color([u8; 3]),
    FixtureQuery(FixtureQuery),
}

impl Value {
    /// [`Value`]の型が与えられた[`Type`]と合致しているか確認
    pub fn matches_type(&self, typ: Type) -> bool {
        match self {
            Self::Duration(_) => matches!(typ, Type::Duration),
            Self::Dimmer(_) => matches!(typ, Type::Dimmer),
            Self::Color(_) => matches!(typ, Type::Color),
            Self::FixtureQuery(_) => matches!(typ, Type::FixtureQuery),
        }
    }

    /// 型を返す
    pub fn typ(&self) -> Type {
        match self {
            Self::Duration(_) => Type::Duration,
            Self::Dimmer(_) => Type::Dimmer,
            Self::Color(_) => Type::Color,
            Self::FixtureQuery(_) => Type::FixtureQuery,
        }
    }

    pub fn unwrap_duration(&self) -> Duration {
        let Self::Duration(val) = self else {
            self.panic_on("unwrap_duration");
        };
        *val
    }

    pub fn unwrap_dimmer(&self) -> u8 {
        let Self::Dimmer(val) = self else {
            self.panic_on("unwrap_dimmer");
        };

        *val
    }

    pub fn unwrap_color(&self) -> [u8; 3] {
        let Self::Color(val) = self else {
            self.panic_on("unwrap_color");
        };

        *val
    }

    pub fn unwrap_query(&self) -> FixtureQuery {
        let Self::FixtureQuery(val) = self else {
            self.panic_on("unwrap_query");
        };

        val.to_owned()
    }

    fn panic_on(&self, method_name: &str) -> ! {
        match self {
            Self::Duration(_) => panic!("Value::{}() is called on Value::Duration", method_name),
            Self::Dimmer(_) => panic!("Value::{}() is called on Value::Dimmer", method_name),
            Self::Color(_) => panic!("Value::{}() is called on Value::Color", method_name),
            Self::FixtureQuery(_) => {
                panic!("Value::{}() is called on Value::FixtureQuery", method_name)
            }
        }
    }
}

/// Queries Fixtures with css-like selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureQuery {
    string: String,
    data: Vec<Selector>,
}

/// [`FixtureQuery`]で指定できるselector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    Id(FixtureId),
    Tags(Vec<FixtureTag>),
}

impl FixtureQuery {
    pub fn from_str(val: impl Into<String>) -> Option<Self> {
        let val = val.into();
        let selector = if val.chars().next()? == '#' {
            Selector::Id(Self::parse_id(val.chars()).expect("todo: Errを返す"))
        } else if val.chars().next().unwrap() == '.' {
            Selector::Tags(Self::parse_tags(&val)?)
        } else {
            return None;
        };
        Some(Self {
            string: val,
            data: vec![selector], // TODO: カンマで複数指定
        })
    }

    /// queryにmatchするFixtureを全て返す
    pub(crate) fn query(&self, doc: DocStateView) -> Vec<FixtureId> {
        self.data.iter().fold(Vec::new(), |mut acc, v| {
            match v {
                Selector::Id(id) => {
                    doc.with_fixtures(|it| {
                        if it.contains_key(id) {
                            acc.push(id.to_owned());
                        } else {
                            warn!(?id, "fixture does not exist");
                        };
                    });
                }
                Selector::Tags(tags) => doc.with_fixtures(|it| {
                    let mut fxts = it
                        .iter()
                        .filter(|(_, fxt)| tags.iter().any(|tag| fxt.tags().contains(tag)))
                        .map(|(fxt_id, _)| fxt_id)
                        .cloned()
                        .collect();
                    acc.append(&mut fxts);
                }),
            }

            acc
        })
    }

    fn parse_id(mut val: impl Iterator<Item = char>) -> Result<FixtureId, uuid::Error> {
        if val.next().unwrap() != '#' {
            unreachable!()
        } else {
            let id: String = val.collect();
            FixtureId::from_str(&id)
        }
    }

    fn parse_tags(val: &str) -> Option<Vec<FixtureTag>> {
        if val.chars().next()? != '.' {
            return None; // TODO: Result::Errのほうがいいかも
        }
        let tags = val.split(".");
        tags.map(|tag| FixtureTag::new(tag)).collect()
    }
}

impl Default for FixtureQuery {
    fn default() -> Self {
        Self {
            string: ".some-tag".to_string(),
            data: vec![Selector::Tags(vec![FixtureTag::new("some-tag").unwrap()])],
        }
    }
}

trait PropsResolver<Id> {
    fn resolve_props(
        &self,
        id: Id,
        given_props: HashMap<String, Value>,
        // fixtures: &FixtureQuery,
    ) -> Box<dyn EffectRuntime>;
}

impl PropsResolver<EffectTemplateId> for DocStateView {
    fn resolve_props(
        &self,
        id: EffectTemplateId,
        given_props: HashMap<String, Value>,
        // fixtures: &FixtureQuery,
    ) -> Box<dyn EffectRuntime> {
        self.with_effect_templates(|it| {
            let tmpl = it.get(&id).unwrap();
            match &tmpl.body {
                EffectTemplateBody::Simple(body) => body.resolve_props(given_props, self.clone()),
                EffectTemplateBody::Sequence(body) => body.resolve_props(given_props, self.clone()),
                EffectTemplateBody::Parallel(body) => body.resolve_props(given_props, self.clone()),
            }
        })
    }
}

impl PropsResolver<&(EffectSpecId, FixtureQuery)> for DocStateView {
    fn resolve_props(
        &self,
        (id, fixtures): &(EffectSpecId, FixtureQuery),
        given_props: HashMap<String, Value>,
        // fixtures: &FixtureQuery,
    ) -> Box<dyn EffectRuntime> {
        self.with_effect_specs(|it| {
            let spec = it.get(id).unwrap();
            match &spec.body {
                EffectSpecBody::Simple(body) => {
                    body.resolve_props(fixtures, given_props, self.clone())
                }
                EffectSpecBody::Sequence(body) => body.resolve_props(given_props, self.clone()),
                EffectSpecBody::Parallel(body) => body.resolve_props(given_props, self.clone()),
            }
        })
    }
}

trait CreateRuntime {
    fn create_runtime(&self, id: EffectId) -> Box<dyn EffectRuntime>;
}

impl CreateRuntime for DocStateView {
    fn create_runtime(&self, id: EffectId) -> Box<dyn EffectRuntime> {
        self.with_effects(|it| {
            let effect = it.get(&id).unwrap();
            match &effect.body {
                EffectBody::Simple(body) => body.create_runtime(self.clone()),
                EffectBody::Sequence(body) => body.create_runtime(self.clone()),
                EffectBody::Parallel(body) => body.create_runtime(self.clone()),
            }
        })
    }
}
