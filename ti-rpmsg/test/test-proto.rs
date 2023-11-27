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

#[test]
fn mk_heartbeat() {
        let buffer: Vec<u8> = [0x12, 0x00].to_vec();

        match msg_uncode(&buffer) {
            EventMsg::Heartbeat() => {println! ("OK heartbeat")},
            _ => panic! ("fail to decode heartbeat")
        }
}