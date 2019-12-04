use super::Backend;
use tremor_script::{errors, query, registry};

#[derive(Debug)]
pub struct TremorQuery {
    registry: registry::Registry,
    aggr_registry: registry::Aggr,
}

impl Default for TremorQuery {
    fn default() -> Self {
        Self {
            registry: registry::registry(),
            aggr_registry: registry::aggr(),
        }
    }
}

impl Backend for TremorQuery {
    fn parse_err(&self, text: &str) -> Option<errors::Error> {
        query::Query::parse(text, &self.registry, &self.aggr_registry).err()
    }
}
