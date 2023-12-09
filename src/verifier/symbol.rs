use std::sync::Mutex;

use lazy_static::lazy_static;
use z3::Symbol;

pub struct SymbolFactory {
    ctr: usize,
    trace: Vec<(&'static str, u32, Option<String>)>,
}

impl SymbolFactory {
    fn new(ctr: usize) -> Self {
        Self {
            ctr,
            trace: Default::default(),
        }
    }

    pub fn add_symbol(&mut self, file: &'static str, line: u32, comment: Option<String>) -> Symbol {
        self.trace.push((file, line, comment));
        self.ctr += 1;
        Symbol::Int((self.ctr - 1) as u32)
    }
}

lazy_static! {
    pub static ref SYMBOL_FACTORY: Mutex<SymbolFactory> = Mutex::new(SymbolFactory::new(0));
}

pub fn get_reason(sym: &Symbol) -> (&'static str, u32, Option<String>) {
    // (filename, line, comment)
    let factory = SYMBOL_FACTORY.lock().unwrap();
    let idx = if let Symbol::Int(idx) = sym {
        *idx as usize
    } else {
        panic!("symbol is not an integer")
    };
    factory.trace.get(idx).unwrap().clone()
}

pub fn get_symbol_count() -> usize {
    let factory = SYMBOL_FACTORY.lock().unwrap();
    factory.ctr
}

macro_rules! symbol {
    ($comment:expr) => {{
        let mut factory = crate::verifier::symbol::SYMBOL_FACTORY.lock().unwrap();
        factory.add_symbol(file!(), line!(), Some($comment.to_string()))
    }};
    () => {{
        let mut factory = crate::verifier::symbol::SYMBOL_FACTORY.lock().unwrap();
        factory.add_symbol(file!(), line!(), None)
    }};
}

pub(crate) use symbol;
