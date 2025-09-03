use std::path::PathBuf;

// Heavily inspired by https://github.com/aQaTL/sane-scan/blob/main/build.rs

fn main() {
    let bindings = bindgen::builder()
        .header("/usr/include/sane/sane.h")
        .rustified_enum("SANE_Unit")
        .rustified_enum("SANE_Value_Type")
        .rustified_enum("SANE_Constraint_Type")
        .rustified_enum("SANE_Action")
        .rustified_enum("SANE_Status")
        .rustified_enum("SANE_Frame")
        .prepend_enum_name(false)
        // .disable_name_namespacing()
        // .disable_nested_struct_naming()
        .derive_debug(true)
        .derive_default(true)
        // .parse_callbacks(Box::new(CamelCaseConverterCallback))
        // .c_naming(false)
        .generate()
        .unwrap();

    bindings
        .write_to_file(
            [std::env::var("OUT_DIR").unwrap().as_str(), "sane.rs"]
                .iter()
                .collect::<PathBuf>(),
        )
        .unwrap();

    println!("cargo:rustc-link-lib=sane");
}
