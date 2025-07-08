use wasm_bindgen::prelude::*;
use mdsvexrs::Context;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Options {
    layout: String,
    custom_tags: Vec<String>,
    // path: String,
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(getter)]
    pub fn layout(&self) -> String {
        self.layout.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_layout(&mut self, layout: String) {
        self.layout = layout;
    }

    #[wasm_bindgen]
    pub fn add_custom_tag(&mut self, tag: String) {
        self.custom_tags.push(tag);
    }
}

#[wasm_bindgen]
pub fn get_default_options() -> Options {
    Options {
        layout: String::new(),
        custom_tags: Vec::new(),
    }
}

#[wasm_bindgen]
pub fn render(contents: &str, opts: &Options) -> String {
    Context::new(mdsvexrs::MdsvexrsOptions {
        layout: opts.layout.to_string(),
        custom_tags: opts.custom_tags.to_vec(),
        // path: options.path,
    })
    .convert(contents)
}
