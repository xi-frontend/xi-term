use operation::Operation;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Update {
    pub rev: Option<u64>,
    #[serde(rename = "ops")] pub operations: Vec<Operation>,
    pub pristine: bool,
}


#[test]
fn deserialize_update() {
    use serde_json;
    let s = r#"{"ops":[{"n":60,"op":"invalidate"},{"lines":[{"cursor":[0],"styles":[],"text":"Bar"},{"styles":[],"text":"Foo"}],"n":12,"op":"ins"}],"pristine":true}"#;
    let update = Update {
        operations: vec![
            Operation {
                operation_type: OperationType::Invalidate,
                nb_lines: 60,
                lines: None,
            },
            Operation {
                operation_type: OperationType::Insert,
                nb_lines: 12,
                lines: Some(vec![
                    Line {
                        cursor: Some(vec![0]),
                        styles: Some(vec![]),
                        text: Some("Bar".to_owned()),
                    },
                    Line {
                        cursor: None,
                        styles: Some(vec![]),
                        text: Some("Foo".to_owned()),
                    },
                ]),
            },
        ],
        pristine: true,
        rev: None,
    };
    let deserialized: Result<Update, _> = serde_json::from_str(s);
    assert_eq!(deserialized.unwrap(), update);
}
