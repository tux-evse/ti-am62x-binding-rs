/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * for test run 'clear && cargo test jsonc'
 * ----------------------------------------
 * start test => cargo test --lib -- --exact
 *
 */

// Attention pour simplifier l'écriture des test le séparateur '\i' est remplacé par '|'
use crate::prelude::*;
use std::mem::MaybeUninit;

extern "C" {
    pub fn memcpy(
        __dst: *mut ::std::os::raw::c_void,
        __src: *const ::std::os::raw::c_void,
        __nbytes: usize,
    );
}

#[test]
fn check_heartbeat() {
    let buffer: [u8; 2] = [0x12, 0x00]; // low_to_high heartbeat

    match msg_uncode(&buffer) {
        EventMsg::Heartbeat() => {
            println!("OK heartbeat")
        }
        _ => panic!("fail to decode heartbeat"),
    }
}

#[test]
fn capi_get_heartbeat() {
    let src: [u8; 2] = [0x12, 0x0]; // low_to_high heartbeat

    // initialized buffer
    #[allow(invalid_value)]
    let buffer: [u8; 256 as usize] = unsafe { MaybeUninit::uninit().assume_init() };

    // simulate C low level read
    unsafe {
        memcpy(
            &buffer as *const _ as *mut ::std::os::raw::c_void,
            src.as_ptr() as *const ::std::os::raw::c_void,
            buffer.len(),
        )
    };

    // shorten receiving buffer to source size
    let data = &buffer[0..src.len()];
    println!("receive data={:#X?}", data);

    // assert buffer match heartbeat encoding
    match msg_uncode(data) {
        EventMsg::Heartbeat() => {
            println!("OK data == heartbeat")
        }
        _ => panic!("fail to decode heartbeat"),
    }
}
