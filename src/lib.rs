mod utils;

use wasm_bindgen::prelude::*;
use std::fmt;
use std::mem;
use std::convert::TryInto;

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
    unsafe {
        alert("Hello, numcrab!");
    }
}

#[wasm_bindgen]
pub fn enable_logging() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub enum Generic {
    Int8, Int16, Int32, Float64
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Dtype {
    alignment: u32,
    item_size: u32,
    kind: char,
}

#[wasm_bindgen]
impl Dtype {
    pub fn new(obj: Generic, align: Option<bool>, copy: Option<bool>) -> Dtype {
        match obj {
            Generic::Int8 => Dtype {
                alignment: 1,
                item_size: 1,
                kind: 'i',
            },
            Generic::Int16 => Dtype {
                alignment: 2,
                item_size: 2,
                kind: 'i',
            },
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
        fmt::write(f, format_args!("dtype('{}{}')", self.kind, self.item_size))
    }
}

const MAX_DIM: i32 = 32;
#[wasm_bindgen]
pub struct NdArray {
    data: Vec<u8>,
    dtype: Dtype,
    shape: Vec<u32>,
}

#[wasm_bindgen]
impl NdArray {
    pub fn new(shape: Vec<u32>, dtype: Option<Dtype>, buffer: Option<Vec<u8>>) -> NdArray {
        let num_els: u32 = shape.iter().product();
        let dtype = match dtype {
            Some(dtype) => dtype,
            None => Dtype::new(Generic::Float64, None, None),
        };
        let data = match buffer {
            Some(v) => v,
            None => Vec::<u8>::with_capacity((num_els * dtype.item_size) as usize)
        };
        NdArray {
            data,
            dtype,
            shape,
        }
    }

    pub fn dtype(&self) -> Dtype {
        self.dtype
    }

    pub fn shape(&self) -> String {
        format!("{:?}", self.shape)
    }

    pub fn pretty_print(&self) -> String {
        let mut res = String::from("[");

        for i in 0..self.data.len() {
            if i % self.dtype.item_size as usize == 0 {
                // add comma in between values (except the first)
                if i != 0 {
                    res.push_str(", ");
                }

                match self.dtype {
                    Dtype {kind: 'i', alignment: _, item_size: 4} =>
                        res.push_str(&i32::from_le_bytes(self.data[i..i + 4].try_into().expect("aarggg line 122")).to_string()),
                    Dtype {kind: 'f', alignment: _, item_size: 8} =>
                        res.push_str(&f64::from_le_bytes(self.data[i..i + 8].try_into().expect("very tired")).to_string()),
                    _ => panic!("Not implemented yet")
                }
            }
        }
        res.push_str("]");
        res
    }
}

#[wasm_bindgen]
pub fn get_shape(val: &JsValue) -> Vec<u32> {
    if !js_sys::Array::is_array(val) {
        return vec![];
    }
    let arr = js_sys::Array::from(val);
    let mut shape= vec![arr.length()];
    shape.extend(&get_shape(&arr.get(0)));
    shape
}

#[wasm_bindgen]
pub fn array(array: js_sys::Array, dtype: Option<Dtype>) -> NdArray {
    let mut buffer: Vec<u8> = vec![];
    // determine dtype
    let mut non_int_found = false;
    let flattened_array = array.flat(MAX_DIM);
    flattened_array.for_each(
        &mut |val: JsValue, idx: u32, _arr: js_sys::Array| {
            match val.as_f64() {
                Some(x) => {
                    if (x % 1_f64 > 1e-16) {non_int_found = true;}
                },
                None => panic!("Non-numerical data is not yet implemented")
            }
        }
    );
    let dtype: Dtype = match dtype {
        Some(dtype) => dtype,
        None => if non_int_found {Dtype::new(Generic::Float64, None, None)} else {Dtype::new(Generic::Int32, None, None) },
    };

    // fill buffer
    flattened_array.for_each(
        &mut |val: JsValue, idx: u32, _arr: js_sys::Array| {
            match dtype {
                Dtype {kind: 'i', alignment, item_size: 4} => {
                    let mut bytes = [0u8; mem::size_of::<i32>()];
                    let converted: i32 = val.as_f64().unwrap() as i32;
                    buffer.extend(converted.to_le_bytes().iter());
                },
                Dtype {kind: 'f', alignment, item_size: 8} => {
                    let mut bytes = [0u8; mem::size_of::<f64>()];
                    let converted = val.as_f64().unwrap();
                    buffer.extend(converted.to_le_bytes().iter());
                },
                _ => panic!("Non-integer types have not yet been implemented")
            };
        }
    );

    NdArray::new(get_shape(&array), Some(dtype), Some(buffer))
}
