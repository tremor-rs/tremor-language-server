use crate::language::prelude::*;
use tremor_script::script::Script;

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
    fn parse_err(&self, text: &str) -> Option<Error> {
        Script::parse(text, &self.registry).err()
    }

    fn functions(&self, module_name: &str) -> Vec<String> {
        if let Some(module) = self.registry.functions.get(module_name) {
            module.keys().cloned().collect()
        } else {
            vec![]
        }
    }
}
