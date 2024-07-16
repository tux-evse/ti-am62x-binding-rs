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

use afbv4::prelude::*;
use prost::Message;

mod pbuf {
    #![allow(non_snake_case)]
    include!("_ti-am62x-evse.rs");
}

// make public internal Iec61851Event
pub type Iec61851Event = pbuf::Iec61851Event;
pub type PwmState= pbuf::PwmState;
pub type SlacState= pbuf::SlacState;

pub enum EventMsg {
    Evt(Iec61851Event),
    Heartbeat(),
    Err(AfbError),
}

pub fn mk_disable() -> Result<Vec<u8>, AfbError> {
    let msg = pbuf::HighToLow {
        message: Some(pbuf::high_to_low::Message::Disable(pbuf::Empty {})),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!("encoding-enable-fail", "{}", error),
    }
}

pub fn mk_enable() -> Result<Vec<u8>, AfbError> {
    let msg = pbuf::HighToLow {
        message: Some(pbuf::high_to_low::Message::Enable(pbuf::Empty {})),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!("encoding-enable-fail", "{}", error),
    }
}

pub fn mk_power(allow: bool) -> Result<Vec<u8>, AfbError> {
    let msg = pbuf::HighToLow {
        message: Some(pbuf::high_to_low::Message::AllowPowerOn(allow)),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!("encoding-power-fail", "{}", error),
    }
}

pub fn mk_heartbeat() -> Result<Vec<u8>, AfbError> {
    let msg = pbuf::HighToLow {
        message: Some(pbuf::high_to_low::Message::Heartbeat(pbuf::CpuHeartbeat {})),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!(
            "encoding-heartbeat-fail",
            "{}", error
        ),
    }
}

// for test only
pub fn mk_lowbeat() -> Result<Vec<u8>, AfbError> {
    let msg = pbuf::LowToHigh {
        message: Some(pbuf::low_to_high::Message::Heartbeat(pbuf::McuHeartbeat{})),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!(
            "encoding-heartbeat-fail",
            "{}", error
        ),
    }
}


pub fn mk_pwm(state: &PwmState, duty_cycle: f32) -> Result<Vec<u8>, AfbError> {

    let msg = pbuf::HighToLow {
        message: Some(pbuf::high_to_low::Message::SetPwm(pbuf::SetPwm {state: *state as i32, duty_cycle})),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!("encoding-setpwm-fail", "{}", error),
    }
}

pub fn mk_slac(state: &SlacState) -> Result<Vec<u8>, AfbError> {

    let msg = pbuf::HighToLow {
        message: Some(pbuf::high_to_low::Message::SetSlac(pbuf::SetSlac {state: *state as i32})),
    };
    let mut buffer = Vec::with_capacity(msg.encoded_len());
    match msg.encode(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => afb_error!("encoding-setpwm-fail", "{}", error),
    }
}

// decode message from encoded buffer
pub fn msg_uncode(buffer: &[u8]) -> EventMsg {
    match pbuf::LowToHigh::decode(buffer) {
        Err(error) => EventMsg::Err(AfbError::new("decoding-buffer-error", 0, format!("{}", error))),
        Ok(data) => match data.message {
            None => EventMsg::Err(AfbError::new("decoding-buffer-empty", 0, "no data to decode")),
            Some(msg) => match msg {
                pbuf::low_to_high::Message::Heartbeat(_) => EventMsg::Heartbeat(),
                pbuf::low_to_high::Message::Event(value) => match Iec61851Event::try_from(value) {
                    Ok(iec) => EventMsg::Evt(iec),
                    Err(error) => EventMsg::Err(AfbError::new(
                        "decoding-ioc6185-error", 0,
                        format!("unknown iec6185 value={} error={}", value, error),
                    )),
                },
            },
        },
    }
}
