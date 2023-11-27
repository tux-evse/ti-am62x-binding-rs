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
use std::mem::MaybeUninit;
use crate::prelude::*;

extern "C" {
    pub fn memcpy(
        __dst: *mut ::std::os::raw::c_void,
        __src: *const ::std::os::raw::c_void,
        __nbytes: usize,
    );
}

#[test]
fn mk_heartbeat() {
        let buffer: [u8;2]= [0x12,0x00];

        match msg_uncode(&buffer) {
            EventMsg::Heartbeat() => {println! ("OK heartbeat")},
            _ => panic! ("fail to decode heartbeat")
        }
}

#[test]
fn test_buffer() {
        let src: [u8;2]= [0x20,0x30];

        #[allow(invalid_value)]
        let buffer: [u8; 256 as usize] = unsafe { MaybeUninit::uninit().assume_init() };
        unsafe { memcpy(&buffer as *const _ as *mut ::std::os::raw::c_void , src.as_ptr() as *const ::std::os::raw::c_void, 256)};
        println! ("buffer({})=[{:#02x},{:#02x}]", buffer.len(),buffer[0], buffer[1]);
}
