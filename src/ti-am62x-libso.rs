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
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Reference: https://github.com/PionixPublic/ti-am62x-evse-sdk.git
 *  following code is a RUST an API version of Pionix ti-am62x-evse-sdk user space module
 *  interfacing through kernel RPMSG the firmware running in the MCU/M4 cortex.
 */

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]

extern crate libafb;
extern crate serde;

#[path = "rpmsg-capi/ti-rpmsg-mod.rs"]
mod rpmsg;

#[path = "protobuf/ti-am62x-codec.rs"]
mod codec;

#[path = "afb-binding/ti-am62x-verbs.rs"]
mod verbs;

#[path = "afb-binding/ti-am62x-binding.rs"]
mod binding;

pub(crate) mod prelude {
    pub(crate) use crate::rpmsg::*;
    pub(crate) use crate::codec::*;
    pub(crate) use crate::verbs::*;
    pub(crate) use crate::binding::*;
}