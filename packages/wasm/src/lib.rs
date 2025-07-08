use wasm_bindgen::prelude::*;
use mdsvexrs::Context;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// pub struct Options {
//     layout: String,
//     // path: String,
// }

// #[wasm_bindgen]
// impl Options {
//     #[wasm_bindgen(getter)]
//     pub fn layout(&self) -> String {
//         self.layout.clone()
//     }

//     #[wasm_bindgen(setter)]
//     pub fn set_layout(&mut self, layout: String) {
//         self.layout = layout;
//     }
// }

// #[wasm_bindgen]
// pub fn get_default_options() -> Options {
//     Options {
//         layout: String::new(),
//     }
// }

#[wasm_bindgen]
pub fn render(contents: &str, layout: &str) -> String {
    Context::new(mdsvexrs::MdsvexrsOptions {
        layout: layout.to_string(),
        // path: options.path,
    })
    .convert(contents)
}
