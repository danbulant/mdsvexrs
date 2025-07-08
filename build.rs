use syntect::{dumps::dump_to_uncompressed_file, parsing::SyntaxSet};

fn main() {
    println!("cargo:rerun-if-changed=assets");
    let mut ss = SyntaxSet::load_defaults_newlines().into_builder();
    ss.add_from_folder("assets", true).unwrap();
    let ss = ss.build();
    
    dump_to_uncompressed_file(&ss, "target/syntax_cache.packdump").expect("Failed to dump syntax cache");
}