use crate::language::prelude::*;
use tremor_script::query::Query;

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

impl Language for TremorQuery {
    fn parse_err(&self, text: &str) -> Option<Error> {
        Query::parse(text, &self.registry, &self.aggr_registry).err()
    }
}
