use super::Language;
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

impl Language for TremorScript {
    fn parse_err(&self, text: &str) -> Option<errors::Error> {
        script::Script::parse(text, &self.registry).err()
    }

    fn functions(&self, module_name: &str) -> Vec<String> {
        if let Some(module) = self.registry.functions.get(module_name) {
            module.keys().cloned().collect()
        } else {
            vec![]
        }
    }
}
