use std::path::Path;

fn main() {
    let udl_path = Path::new("src/mopro.udl");
    uniffi::generate_scaffolding(udl_path.to_str().unwrap()).unwrap();
}
