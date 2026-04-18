use tsukuyomidmx_core::{
    doc::{Doc, FakeFixtureDefRegistry},
    effects::{EffectSpec, EffectSpecId, EffectTemplate, EffectTemplateBody, EffectTemplateId},
};

#[test]
fn sequence_function_works() {
    /*let mut def_rg = FakeFixtureDefRegistry::new();
    let doc = Doc::new_with_def_registry(Box::new(def_rg));

    let spec = EffectSpec {
        id: EffectSpecId::new(),
        name: "blink".into(),
        body: EffectSpecBody::Sequence(SequenceEffectSpecBody {
            props: vec![
                ("duration", Type::Duration),
                ("color", Type::Color),
                ("fixtures", Type::FixtureQuery),
            ]
            .into_iter()
            .map(|(name, typ)| (name.to_string(), typ))
            .collect(),
            steps: vec![
                SequenceSpecStep {
                    hold: Expression::Prop("duration".into()),
                    fade_in: Expression::Value(Value::Duration(Duration::ZERO)),
                    body: EffectBodyOrReference::Body(EffectSpecBody::Simple(
                        SimpleEffectSpecBody {
                            dimmer: Some(Expression::Value(Value::Dimmer(255))),
                            color: Some(Expression::Prop("color".into())),
                        },
                    )),
                },
                SequenceSpecStep {
                    hold: Expression::Prop("duration".into()),
                    fade_in: Expression::Value(Value::Duration(Duration::ZERO)),
                    body: EffectBodyOrReference::Body(EffectSpecBody::Simple(
                        SimpleEffectSpecBody {
                            dimmer: Some(Expression::Value(Value::Dimmer(0))),
                            color: Some(Expression::Prop("color".into())),
                        },
                    )),
                },
            ],
        }),
    };

    let tmpl = EffectTemplate {
        id: EffectTemplateId::new(),
        name: "red-blink-on-left".into(),
        body: EffectTemplateBody::Sequence(SequenceEffectTemplateBody::FromSpec {
            spec_id: spec.id(),
            spec_props: HashMap::from([
                ("color".into(), Expression::Value(Value::Color([255, 0, 0]))),
                (
                    "fixtures".into(),
                    Expression::Value(Value::FixtureQuery(
                        FixtureQuery::from_str(".left").unwrap(),
                    )),
                ),
            ]),
            props: HashMap::from([("duration".into(), Type::Duration)]),
        }),
    };

    //assert_eq!(EffectTemplate::from_spec(spec.id(), ".left"), tmpl);

    let fx = Effect {
        id: EffectId::new(),
        name: "red-blink-on-left-500ms".into(),
        body: EffectBody::Sequence(SequenceEffectBody::FromTemplate(
            tmpl.id,
            HashMap::from([(
                "duration".into(),
                Value::Duration(Duration::from_millis(500)),
            )]),
        )),
    };

    let rt = fx.body.create_runtime(doc.state_view());*/
}
