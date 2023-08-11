/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/
extern crate bindgen;

fn main() {
    let proto_path="src/protobuf";
    println!("cargo:rerun-if-changed=src/rpmsg-capi/ti-rpmsg-map.h");
    // following lines prevent cargo to fail when generated target files are not present. Unfortunately they rebuild wrappers at each <cargo build> :(
    //println!("cargo:rerun-if-changed=src/protobuf/_ti-am62x-evse.rs");
    //println!("cargo:rerun-if-changed=src/rpmsg-capi/_ti-rpmsg-map.rs");

    // generate protobuf encoder/decoder
    prost_build::Config::new()
        .default_package_filename("_ti-am62x-evse")
        .out_dir(proto_path)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        //.type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile_protos(&[[proto_path, "high_to_low.proto"].join("/"),[proto_path,"low_to_high.proto"].join("/")], &[proto_path])
        // https://github.com/dflemstr/prost-simple-rpc
        //.service_generator(Box::new(prost_simple_rpc_build::ServiceGenerator::new())) //
        .expect("Fail to generate protobus protobuf");

    // force libti_rpmsg_char search
    println!("cargo:rustc-link-search=/usr/local/lib64");
    println!("cargo:rustc-link-search=/usr/local/lib");
    println!("cargo:rustc-link-arg=-lti_rpmsg_char");

    let header = "
    // -----------------------------------------------------------------------
    //         <- private 'librpmg' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs for C/Rust glue options
    //     - src/rpmsg-capi/librpmg_char.h for C prototype inputs
    // -----------------------------------------------------------------------
    ";
    let librpmg = bindgen::Builder::default()
        .header("src/rpmsg-capi/ti-rpmsg-map.h") // C prototype wrapper input
        .raw_line(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("rpmsg_char_.*")
        .allowlist_function("write")
        .allowlist_function("read")
        .allowlist_type("rpmsg_char_.*")
        .generate()
        .expect("Unable to generate librpmg");

    librpmg
        .write_to_file("src/rpmsg-capi/_ti-rpmsg-map.rs")
        .expect("Couldn't write librpmg!");
}
