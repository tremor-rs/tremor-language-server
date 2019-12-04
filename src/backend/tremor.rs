use super::Backend;
use tremor_script::{errors, registry, script};

#[derive(Debug)]
pub struct TremorScript {
    registry: registry::Registry,
}

impl Default for TremorScript {
    fn default() -> Self {
        Self {
            registry: registry::registry(),
        }
    }
}

impl Backend for TremorScript {
    fn parse_err(&self, text: &str) -> Option<errors::Error> {
        script::Script::parse(text, &self.registry).err()
    }
}
