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

use crate::prelude::*;
use afbv4::prelude::*;
use rpmsg::prelude::*;

pub(crate) struct ApiUserData {
    pub uid: &'static str,
    pub devname: Option<&'static str>,
    pub eptname: &'static str,
    pub eptnum: i32,
    pub tic: u32,
}

fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

impl AfbApiControls for ApiUserData {
    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);

        Ok(())
    }

    // mandatory for downcasting back to custom api data object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    let uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "ti-am62x"
    };

    let apiname = if let Ok(value) = jconf.get::<String>("api") {
        to_static_str(value)
    } else {
        "ti-rmsg"
    };

    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    let devname = if let Ok(value) = jconf.get::<String>("devname") {
        Some(to_static_str(value))
    } else {
        None
    };

    let socname = if let Ok(value) = jconf.get::<String>("socname") {
        Some(to_static_str(value))
    } else {
        None
    };

    let eptname = if let Ok(value) = jconf.get::<String>("eptname") {
        to_static_str(value)
    } else {
        "tux-evse-rmsg"
    };

    let eptnum = if let Ok(value) = jconf.get::<i32>("ept_num") {
        value
    } else {
        14
    };

    let tic = if let Ok(value) = jconf.get::<u32>("ept_tic") {
        value
    } else {
        250
    };

    let permission = if let Ok(value) = jconf.get::<String>("permission") {
        AfbPermission::new(to_static_str(value))
    } else {
        AfbPermission::new("acl:rmsg:ti")
    };

    let config = ApiUserData {
        uid,
        devname,
        eptname,
        eptnum,
        tic,
    };

    // initialization of ti rpm_char_lib should be done once at initialization
    ti_init(socname)?;

    // create a new api
    let api = AfbApi::new(apiname)
        .set_info(info)
        .set_permission(permission);

    // register verbs and events
    register(api, &config)?;

    // finalize api
    Ok(api.finalize()?)
}

// register binding within afbv4
AfbBindingRegister!(binding_init);