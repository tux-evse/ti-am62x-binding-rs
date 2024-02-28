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
use typesv4::prelude::*;

pub(crate) struct ApiUserData {
    pub uid: &'static str,
    pub cdev: Option<&'static str>,
    pub eptname: &'static str,
    pub lock_api: &'static str,
    pub lock_verb: &'static str,
    pub rport: i32,
    pub tic: u32,
}

fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

struct ApiCtxData{}

impl AfbApiControls for ApiCtxData {

    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        // place here any required api subscription
        afb_log_msg!(Debug, None, "start apiv4={:?}", api.get_apiv4());
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
    // register custom afb-v4 type converter
    rpmsg_register()?;
    am62x_registers()?;
    slac_registers()?;

    let uid = to_static_str(jconf.get::<String>("uid")?);
    let api = jconf.default::<&'static str>("api",uid)?;
    let info = jconf.default::<&'static str>("info","")?;
    let cdev = jconf.optional::<&'static str>("cdev")?;
    let socname = jconf.optional::<&'static str>("socname")?;
    let eptname = jconf.default::<&'static str>("eptname","tux-evse-rmsg")?;
    let rport = jconf.default::<i32>("rport", 14)?;
    let tic = jconf.default::<u32>("tic", 5000)?;
    let lock_api = jconf.get::<&'static str>("lock_api")?;
    let lock_verb = jconf.get::<&'static str>("lock_verb")?;

    let config = ApiUserData {
        uid,
        cdev,
        rport,
        eptname,
        tic,
        lock_api,
        lock_verb,
    };

    // initialization of ti rpm_char_lib should be done once at initialization
    ti_init(socname)?;

    // create a new api
    let api = AfbApi::new(api)
        .set_info(info)
        .set_callback(Box::new(ApiCtxData{}));

    if let Ok(value) = jconf.get::<String>("permission") {
        api.set_permission(AfbPermission::new(to_static_str(value)));
    };

    // register verbs and events
    register(rootv4, api, &config)?;

    // finalize api
    api.require_api(lock_api);
    let api= api.finalize()?;

    Ok(api)
}

// register binding within afbv4
AfbBindingRegister!(binding_init);
