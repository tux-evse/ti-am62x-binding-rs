/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 */

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]

#[cfg(not(afbv4))]
extern crate afbv4;

#[path = "../capi/capi-mod.rs"]
mod capi;

#[path = "../protobuf/ti-am62x-codec.rs"]
mod codec;

#[path = "ti-rpmsg.rs"]
mod rpmsg;

pub mod prelude {
    pub use crate::capi::*;
    pub use crate::codec::*;
    pub use crate::rpmsg::*;
}