use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=AUGUR_RELEASE");

    if env::var_os("AUGUR_RELEASE").is_none() {
        println!("cargo:rustc-env=AUGUR_RELEASE=dev");
    }
}
