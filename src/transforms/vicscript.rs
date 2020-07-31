use super::Transform;
use crate::{
    event::Event,
    internal_events::{VicscriptEventProcessed, VicscriptFailedMapping},
    topology::config::{DataType, TransformConfig, TransformContext, TransformDescription},
    vicscript::{parser::parse as parse_mapping, Mapping},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Derivative)]
#[serde(deny_unknown_fields, default)]
#[derivative(Default)]
pub struct VicscriptConfig {
    pub mapping: String,
    pub drop_on_err: bool,
}

inventory::submit! {
    TransformDescription::new::<VicscriptConfig>("vicscript")
}

#[typetag::serde(name = "vicscript")]
impl TransformConfig for VicscriptConfig {
    fn build(&self, _cx: TransformContext) -> crate::Result<Box<dyn Transform>> {
        Ok(Box::new(Vicscript::new(self.clone())?))
    }

    fn input_type(&self) -> DataType {
        DataType::Log
    }

    fn output_type(&self) -> DataType {
        DataType::Log
    }

    fn transform_type(&self) -> &'static str {
        "vicscript"
    }
}

#[derive(Debug)]
pub struct Vicscript {
    mapping: Mapping,
    drop_on_err: bool,
}

impl Vicscript {
    pub fn new(config: VicscriptConfig) -> crate::Result<Vicscript> {
        Ok(Vicscript {
            mapping: parse_mapping(&config.mapping)?,
            drop_on_err: config.drop_on_err,
        })
    }
}

impl Transform for Vicscript {
    fn transform(&mut self, mut event: Event) -> Option<Event> {
        emit!(VicscriptEventProcessed);

        if let Err(err) = self.mapping.execute(&mut event) {
            error!(message = "mapping failed", %err);
            emit!(VicscriptFailedMapping { error: err });

            if self.drop_on_err {
                return None;
            }
        }

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use string_cache::DefaultAtom as Atom;

    #[test]
    fn check_vicscript_adds() {
        let event = {
            let mut event = Event::from("augment me");
            event.as_mut_log().insert("copy_from", "buz");
            event
        };

        let conf = VicscriptConfig {
            mapping: r#".foo = "bar"
            .bar = "baz"
            .copy = .copy_from"#
                .to_string(),
            drop_on_err: true,
        };
        let mut tform = Vicscript::new(conf).unwrap();

        let result = tform.transform(event.clone()).unwrap();
        assert_eq!(
            result
                .as_log()
                .get(&Atom::from("message"))
                .unwrap()
                .to_string_lossy(),
            "augment me"
        );
        assert_eq!(
            result
                .as_log()
                .get(&Atom::from("copy_from"))
                .unwrap()
                .to_string_lossy(),
            "buz"
        );
        assert_eq!(
            result
                .as_log()
                .get(&Atom::from("foo"))
                .unwrap()
                .to_string_lossy(),
            "bar"
        );
        assert_eq!(
            result
                .as_log()
                .get(&Atom::from("bar"))
                .unwrap()
                .to_string_lossy(),
            "baz"
        );
        assert_eq!(
            result
                .as_log()
                .get(&Atom::from("copy"))
                .unwrap()
                .to_string_lossy(),
            "buz"
        );
    }
}
