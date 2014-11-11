use payload::Payload;
use ast;

mod rust;

pub trait Language {
    fn setup(&self) -> ast::Statement { ast::Noop }
    fn announce(&self) -> ast::Statement { ast::Noop }
    fn install(&self) -> ast::Statement { ast::Noop }
    fn script(&self) -> ast::Statement { ast::Noop }
}

pub fn for_payload<L: Language>(payload: &Payload) -> L {
    match payload.config.language.as_slice() {
        "rust" => rust::Rust(payload),
        _ => unimplemented!(),
    }
}
