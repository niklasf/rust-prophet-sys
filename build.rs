use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    bindgen::builder()
        .layout_tests(false)
        .header("prophet_tb_gen_and_probe/src/prophet.h")
        .allowlist_function("prophet_.*")
        .allowlist_type("prophet_.*")
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .flag_if_supported("-Wno-unknown-pragmas")
        .include("prophet_tb_gen_and_probe/src")
        .file("prophet_tb_gen_and_probe/src/bitboard.cpp")
        .file("prophet_tb_gen_and_probe/src/compressed_tb.cpp")
        .file("prophet_tb_gen_and_probe/src/eg_movegen.cpp")
        .file("prophet_tb_gen_and_probe/src/eg_position.cpp")
        .file("prophet_tb_gen_and_probe/src/egtb.cpp")
        .file("prophet_tb_gen_and_probe/src/egtb_ids.cpp")
        .file("prophet_tb_gen_and_probe/src/kkx.cpp")
        .file("prophet_tb_gen_and_probe/src/linearize.cpp")
        .file("prophet_tb_gen_and_probe/src/prophet.cpp")
        .file("prophet_tb_gen_and_probe/src/triangular_indexes.cpp")
        .file("prophet_tb_gen_and_probe/src/uci.cpp")
        .compile("prophet");

    println!("cargo:root={}", out_dir.display());
    println!(
        "cargo:include={}",
        env::current_dir()
            .unwrap()
            .join("prophet_tb_gen_and_probe")
            .join("src")
            .display()
    );

    println!("cargo:rustc-link-lib=zstd");
}
