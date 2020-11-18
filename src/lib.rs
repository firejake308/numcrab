mod utils;

use wasm_bindgen::prelude::*;
use std::fmt;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, numcrab!");
}

#[wasm_bindgen]
pub enum Generic {
    Int32, Float64
}

#[wasm_bindgen]
pub struct Dtype {
    alignment: usize,
    item_size: usize,
    kind: char,
}

#[wasm_bindgen]
impl Dtype {
    pub fn new(obj: Generic, align: Option<bool>, copy: Option<bool>) -> Dtype {
        match obj {
            Generic::Int32 => Dtype {
                        alignment: 4,
                        item_size: 4,
                        kind: 'i',
                    },
            Generic::Float64 => Dtype {
                        alignment: 8,
                        item_size: 8,
                        kind: 'f',
                    },
        }
    }

    pub fn pretty_print(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Dtype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::write(f, format_args!("{}{}", self.kind, self.item_size))
    }
}
