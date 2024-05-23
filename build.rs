use std::env;
use std::path::{PathBuf};

const C_HEADER_OUTPUT: &str = "ggrsc.h";

// Environment variable name prefixes worth including for diags
const ENV_PATTERNS: &[&str] = &["CARGO_", "RUST", "LIB"];

fn main() -> Result<(), &'static str> {
    eprintln!("build.rs command line: {:?}", std::env::args());
    eprintln!("Environment:");
    std::env::vars()
        .filter(|(k, _)| ENV_PATTERNS.iter().any(|prefix| k.starts_with(prefix)))
        .for_each(|(k, v)| eprintln!("  {}={:?}", k, v));

    // We only want to generate bindings for `cargo build`, not `cargo test`.
    // FindRust.cmake defines $CARGO_CMD so we can differentiate.
    let cargo_cmd = env::var("CARGO_CMD").unwrap_or_else(|_| "".into());

    // If this environmment variable chnages, we should re-run this script.
    println!("cargo:rerun-if-env-changed=LIBDEMO");

    match cargo_cmd.as_str() {
        "build" => {
            // Generate C bindings as a part of the build.
            generate_c_bindings()?;
        }

        _ => {
            return Ok(());
        }
    }

    Ok(())
}

/// Use cbindgen to generate C-headers for Rust library.
fn generate_c_bindings() -> Result<(), &'static str> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").or(Err("CARGO_MANIFEST_DIR not specified"))?;
    let build_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| ".".into()));
    let outfile_path = build_dir.join(C_HEADER_OUTPUT);

    // Useful for build diagnostics
    eprintln!("cbindgen outputting {:?}", &outfile_path);
    cbindgen::generate(crate_dir)
        .expect("Unable to generate C headers for Rust code")
        .write_to_file(&outfile_path);

    Ok(())
}

