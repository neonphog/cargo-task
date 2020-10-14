const CARGO_TOML_PATH: &str = "./Cargo.toml";
const VER_FILE_PATH: &str = "./ver.rs";
const BUILD_RS_PATH: &str = "./build.rs";

/// Generate the ver.rs file in OUT_DIR containing CARGO_TASK_VER constant.
pub fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let ver_file = std::path::Path::new(&out_dir).join(VER_FILE_PATH);
    println!("cargo:rerun-if-changed={}", BUILD_RS_PATH);
    println!("cargo:rerun-if-changed={}", CARGO_TOML_PATH);

    let cargo_toml = std::fs::read(CARGO_TOML_PATH).unwrap();
    let cargo_toml = String::from_utf8_lossy(&cargo_toml);

    for line in cargo_toml.split('\n') {
        if line.starts_with("version = ") {
            let idx1 = line.find('"').unwrap();
            let idx2 = line.rfind('"').unwrap();

            let ver = &line[idx1 + 1..idx2];

            std::fs::write(
                &ver_file,
                format!(
                    "/// Cargo Task Version\npub const CARGO_TASK_VER: &str = \"{}\";\n",
                    ver,
                ),
            )
            .unwrap();

            return;
        }
    }

    panic!("Unable to read version from Cargo.toml");
}
