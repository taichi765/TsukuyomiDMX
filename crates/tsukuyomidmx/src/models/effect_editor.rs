use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use slint::{Model, ModelRc, ModelTracker, SharedString, ToSharedString, VecModel};
use tsukuyomidmx_core::{
    doc::{Doc, DocEffect, DocStateView},
    effects::{Effect, EffectBody, FixtureQuery, SimpleEffectBody},
    prelude::EffectId,
};

use crate::{Observable, app::AnyEffectId, ui};

pub struct EffectEditorModel {
    opening_tabs: Rc<VecModel<ui::EffectEditorPanelData>>,
    current_effect_data: Observable<Option<EffectEditorData>>,
}

impl EffectEditorModel {
    pub fn new(doc: Arc<Mutex<Doc>>, id_state: Observable<Option<AnyEffectId>>) -> Rc<Self> {
        let me = Rc::new(Self {
            opening_tabs: Rc::new(VecModel::from(Vec::new())),
            current_effect_data: Observable::new(None),
        });

        doc.lock().unwrap().subscribe({
            let me = Rc::clone(&me);
            let doc = Arc::clone(&doc);
            let id_state = id_state.clone();

            Box::new(move |ev| match ev {
                DocEffect::EffectUpdated(id) => {
                    let new_data =
                        EffectEditorData::from_effect(*id, doc.lock().unwrap().state_view());

                    me.current_effect_data.set(Some(new_data));
                }
                DocEffect::EffectRemoved(id) => {
                    if let Some(AnyEffectId::Effect(cur_id)) = id_state.get()
                        && *id == cur_id
                    {
                        todo!("現在選択中のeffectが削除されたときの挙動")
                    };

                    let pos = me
                        .opening_tabs
                        .iter()
                        .position(|data| {
                            data.id == serde_json::to_string(id).unwrap().to_shared_string()
                        })
                        .unwrap();
                    me.opening_tabs.remove(pos);
                }
                DocEffect::EffectSpecUpdated(_id) => {
                    todo!()
                }
                DocEffect::EffectSpecRemoved(_id) => {
                    todo!()
                }
                DocEffect::EffectTemplateUpdated(_id) => {
                    todo!()
                }
                DocEffect::EffectTemplateRemoved(_id) => {
                    todo!()
                }
                _ => (),
            })
        });

        id_state.subscribe({
            let me = Rc::clone(&me);
            let doc = doc.lock().unwrap().state_view();

            move |id| {
                if let Some(id) = id {
                    let id_str = serde_json::to_string(id).unwrap().to_shared_string();
                    if !me.opening_tabs.iter().any(|v| v.id == id_str) {
                        me.opening_tabs.push(ui::EffectEditorPanelData {
                            id: id_str,
                            name: doc.get_name(*id).unwrap(),
                        });
                    }
                }

                let new_data = id.map(|id| match id {
                    AnyEffectId::Effect(id) => EffectEditorData::from_effect(id, doc.clone()),
                    AnyEffectId::Spec(_id) => todo!(),
                    AnyEffectId::Template(_id) => todo!(),
                });
                me.current_effect_data.set(new_data);
            }
        });

        me
    }

    /// Returns the [`ModelRc`][slint::ModelRc] of opening tabs;
    pub fn tabs_model(&self) -> ModelRc<ui::EffectEditorPanelData> {
        Rc::clone(&self.opening_tabs).into()
    }

    pub fn subscribe_current_effect_data<F>(&self, f: F)
    where
        F: FnMut(&Option<EffectEditorData>) + 'static,
    {
        self.current_effect_data.subscribe(f);
    }

    /// Closes the tab specified by `idx`.
    pub fn close_tab(&self, idx: usize) {
        self.opening_tabs.remove(idx);
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum EffectEditorData {
    Simple(SimpleEffectSingleViewAdopterData),
    Sequence(),
    Parallel(),
}

impl EffectEditorData {
    fn from_effect(effect_id: EffectId, doc: DocStateView) -> Self {
        doc.with_effects(|it| {
            let effect = it.get(&effect_id).unwrap();
            match effect.body() {
                EffectBody::Simple(body) => match body {
                    SimpleEffectBody::New { fixtures, values } => {
                        let channels = doc.with_fixtures_and_defs(|fxts, defs| {
                            vec![ui::SimpleEffectSingleViewChannelData {
                                enabled: true,
                                kind: ui::ChannelKind::Dimmer,
                                name: "Dimmer".to_shared_string(),
                                offset: 0,
                                value: 255, // TODO
                            }]
                        });
                        EffectEditorData::Simple(SimpleEffectSingleViewAdopterData {
                            fixture_query: fixtures.to_shared_string(),
                            effect_name: effect.name().to_shared_string(),
                            item: ui::SimpleEffectSingleViewItemData {
                                channels: Rc::new(VecModel::from(channels)).into(),
                            },
                        })
                    }
                    SimpleEffectBody::FromTemplate { .. } => todo!(),
                },
                EffectBody::Sequence(_) => todo!(),
                EffectBody::Parallel(_) => todo!(),
            }
        })
    }
}

/// DTO to represent properties in `SimpleEffectSingleViewAdopter`.
#[derive(Debug)]
pub struct SimpleEffectSingleViewAdopterData {
    pub fixture_query: SharedString,
    pub effect_name: SharedString,
    pub item: ui::SimpleEffectSingleViewItemData,
}

trait GetEffectName {
    /// Gets effect-like's name from [`AnyEffectId`].
    fn get_name(&self, id: AnyEffectId) -> Option<SharedString>;
}

impl GetEffectName for DocStateView {
    fn get_name(&self, id: AnyEffectId) -> Option<SharedString> {
        match id {
            AnyEffectId::Effect(id) => {
                self.with_effects(|it| it.get(&id).map(|fx| fx.name().to_shared_string()))
            }
            AnyEffectId::Spec(id) => {
                self.with_effect_specs(|it| it.get(&id).map(|spec| spec.name().to_shared_string()))
            }
            AnyEffectId::Template(id) => self
                .with_effect_templates(|it| it.get(&id).map(|tmpl| tmpl.name().to_shared_string())),
        }
    }
}

#[cfg(test)]
mod tests {}
