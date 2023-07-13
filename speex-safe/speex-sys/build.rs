////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////
use bindgen::CargoCallbacks;
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let c_files = [
        "bits.c",
        "cb_search.c",
        "exc_5_64_table.c",
        "exc_5_256_table.c",
        "exc_8_128_table.c",
        "exc_10_16_table.c",
        "exc_10_32_table.c",
        "exc_20_32_table.c",
        "filters.c",
        "gain_table.c",
        "gain_table_lbr.c",
        "hexc_10_32_table.c",
        "hexc_table.c",
        "high_lsp_tables.c",
        "kiss_fft.c",
        "kiss_fftr.c",
        "lpc.c",
        "lsp.c",
        "lsp_tables_nb.c",
        "ltp.c",
        "modes.c",
        "modes_wb.c",
        "nb_celp.c",
        "quant_lsp.c",
        "sb_celp.c",
        "smallft.c",
        "speex.c",
        "speex_callbacks.c",
        "speex_header.c",
        "stereo.c",
        "vbr.c",
        "vorbis_psy.c",
        "vq.c",
        "window.c",
    ];

    let mut ccomp = cc::Build::new();

    ccomp.include("speex/include");
    println!("cargo:include=speex/include");
    println!("cargo:rustc-link-lib=static=speex");
    let link_dir = dst.join("lib").to_str().map(str::to_string).unwrap();
    println!("cargo:rustc-link-search={link_dir}");
    println!("cargo:static=1");

    for path in c_files {
        ccomp.file(format!("speex/libspeex/{path}"));
    }

    ccomp.define("FLOATING_POINT", None).define("EXPORT", "");
    ccomp.warnings(false);
    ccomp.out_dir(dst.join("lib"));
    ccomp.compile("speex");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Failed to write bindings");
}
