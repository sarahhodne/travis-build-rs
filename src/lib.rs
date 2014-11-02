#![crate_name = "travis_build"]
#![crate_type = "lib"]
#![license = "MIT"]
#![comment = "Travis Build generates build scripts"]

#![experimental]
#![feature(macro_rules)]

extern crate serialize;

pub use script::Script;
pub use payload::Payload;

pub mod ast;
pub mod bash;
pub mod payload;
pub mod script;
pub mod script_templates;
pub mod components;
pub mod test_ast_runner;
