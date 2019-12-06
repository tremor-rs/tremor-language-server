mod prelude;
mod query;
mod script;

pub use tremor_script::pos;

// language names
pub const TREMOR_SCRIPT: &str = "tremor-script";
pub const TREMOR_QUERY: &str = "tremor-query";

// file extensions
pub const TREMOR_SCRIPT_FILE_EXT: &str = "tremor";
pub const TREMOR_QUERY_FILE_EXT: &str = "trickle";

pub trait Language: Send + Sync {
    fn parse_err(&self, text: &str) -> Option<prelude::Error>;

    fn functions(&self, _module_name: &str) -> Vec<String> {
        vec![]
    }
}

pub fn lookup(name: &str) -> Option<Box<dyn Language>> {
    match name {
        TREMOR_SCRIPT | TREMOR_SCRIPT_FILE_EXT => Some(Box::new(script::TremorScript::default())),
        TREMOR_QUERY | TREMOR_QUERY_FILE_EXT => Some(Box::new(query::TremorQuery::default())),
        _ => None,
    }
}
