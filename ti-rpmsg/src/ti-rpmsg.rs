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

use crate::prelude::*;
use afbv4::prelude::*;

// import serde/json converters
AfbDataConverter!(pwm_state_type, PwmState);

pub fn rpmsg_register() -> Result<(), AfbError> {
   // register imported serde type
    pwm_state_type::register()?;


    Ok(())
}