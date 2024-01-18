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
use std::cell::Cell;
use std::mem::MaybeUninit;
use std::rc::Rc;

use crate::prelude::*;
use afbv4::prelude::*;
use rpmsg::prelude::*;
use typesv4::prelude::*;

// protobuf maximum buffer size
const PROTOBUF_MAX_CAPACITY: usize = 256;

// timer ctx and callback
struct DevTimerCtx {
    dev: Rc<TiRpmsg>,
    heartbeat: Vec<u8>,
}
AfbTimerRegister!(TimerCtrl, timer_callback, DevTimerCtx);
fn timer_callback(_timer: &AfbTimer, _decount: u32, ctx: &mut DevTimerCtx) -> Result<(), AfbError> {
    // send heartbeat message
    ctx.dev.write(&ctx.heartbeat)
}

fn process_iec6185(
    apiv4: AfbApiV4,
    iec: &Iec61851Event,
    ctx: &mut DevAsyncCtx,
) -> Result<(), AfbError> {
    let iec_msg = match iec {
        Iec61851Event::CarPluggedIn => {
            // request lock motor from i2c binding
            AfbSubCall::call_sync(apiv4, ctx.lock_api, ctx.lock_verb, "{'action':'on'}")?;
            Iec6185Msg::Plugged(true)
        }

        Iec61851Event::CarUnplugged => {
            let msg = mk_pwm(&PwmState::Off, 0.0)?;
            ctx.dev.write(&msg)?;
            // Fulup TBD for test only unlock motor as soon as IEC-UNLOCK is received
            AfbSubCall::call_sync(apiv4, ctx.lock_api, ctx.lock_verb, "{'action':'off'}")?;
            Iec6185Msg::Plugged(false)
        }

        Iec61851Event::CarRequestedPower => {
            // send request to charging manager authorization
            Iec6185Msg::PowerRqt(ctx.imax)
        }

        Iec61851Event::CarRequestedStopPower => {
            // set max power 0
            // M4 firmware cut power
            ctx.imax = 0;
            Iec6185Msg::PowerRqt(ctx.imax)
        }

        // relay close vehicle charging
        Iec61851Event::PowerOn => {
            // notify max current
            Iec6185Msg::RelayOn(true)
        }

        // relay close vehicle charging
        Iec61851Event::PowerOff => {
            // unlock motor
            AfbSubCall::call_sync(apiv4, ctx.lock_api, ctx.lock_verb, "{'action':'off'}")?;
            Iec6185Msg::RelayOn(false)
        }

        Iec61851Event::ErrorE
        | Iec61851Event::ErrorDf
        | Iec61851Event::ErrorRelais
        | Iec61851Event::ErrorRcd => {
            // no action send error message
            Iec6185Msg::Error(iec.as_str_name().to_string())
        }

        Iec61851Event::PpImax13a => {
            if ctx.imax != 13 {
                afb_log_msg!(Debug, None, "New iec6185:{:?}", iec);
            }
            ctx.imax = 13;
            return Ok(());
        }
        Iec61851Event::PpImax20a => {
            if ctx.imax != 20 {
                afb_log_msg!(Debug, None, "New iec6185:{:?}", iec);
            }
            ctx.imax = 20;
            return Ok(());
        }

        Iec61851Event::PpImax32a => {
            if ctx.imax != 32 {
                afb_log_msg!(Debug, None, "New iec6185:{:?}", iec);
            }
            ctx.imax = 32;
            return Ok(());
        }

        Iec61851Event::PpImax64a => {
            if ctx.imax != 64 {
                afb_log_msg!(Debug, None, "New iec6185:{:?}", iec);
            }
            ctx.imax = 64;
            return Ok(());
        }

        _ => {
            // ignore any other case
            afb_log_msg!(Debug, None, "New ignored:{:?}", iec);
            return Ok(());
        }
    };

    afb_log_msg!(Notice, None, "JobPost push event:{:?}", iec_msg);
    ctx.evt.push(iec_msg);
    Ok(())
}

// on event ctx and callback
struct DevAsyncCtx {
    count: Cell<u32>,
    dev: Rc<TiRpmsg>,
    lock_api: &'static str,
    lock_verb: &'static str,
    imax: u32,
    evt: &'static AfbEvent,
    apiv4: AfbApiV4,
}
AfbEvtFdRegister!(DecAsyncCtrl, async_dev_cb, DevAsyncCtx);
fn async_dev_cb(_event: &AfbEvtFd, revent: u32, ctx: &mut DevAsyncCtx) -> Result<(), AfbError> {
    if revent == AfbEvtFdPoll::IN.bits() {
        #[allow(invalid_value)]
        let mut buffer: [u8; PROTOBUF_MAX_CAPACITY as usize] =
            unsafe { MaybeUninit::uninit().assume_init() };

        let len = ctx.dev.read(&mut buffer)?;
        let data = &buffer[0..len];
        match msg_uncode(data) {
            EventMsg::Err(error) => {
                afb_log_msg!(Critical, None, "{}", error);
            }

            EventMsg::Heartbeat() => {
                let count = ctx.count.get() + 1;
                ctx.count.set(count);
            }

            EventMsg::Evt(iec6185) => {
                process_iec6185(ctx.apiv4, &iec6185, ctx)?;
            }
        }
    }
    Ok(())
}

struct SubscribeData {
    evt: &'static AfbEvent,
}
AfbVerbRegister!(SubscribeCtrl, subscribe_callback, SubscribeData);
fn subscribe_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut SubscribeData,
) -> Result<(), AfbError> {
    let subcription = args.get::<bool>(0)?;

    if subcription {
        ctx.evt.subscribe(request)?;
    } else {
        ctx.evt.unsubscribe(request)?;
    }
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct EnableData {
    dev: Rc<TiRpmsg>,
    enable: Vec<u8>,
    disable: Vec<u8>,
}
AfbVerbRegister!(EnableCtrl, enable_callback, EnableData);
fn enable_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut EnableData,
) -> Result<(), AfbError> {
    let enable = args.get::<bool>(0)?;

    let msg = if enable { &ctx.enable } else { &ctx.disable };
    if let Err(error) = ctx.dev.write(msg) {
        return afb_error!("m4-rpc-fail", "enable({}):{}", enable, error);
    };

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct PowerData {
    dev: Rc<TiRpmsg>,
    enable: Vec<u8>,
    disable: Vec<u8>,
}
AfbVerbRegister!(PowerCtrl, power_callback, PowerData);
fn power_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut PowerData,
) -> Result<(), AfbError> {
    let enable = args.get::<bool>(0)?;

    let msg = if enable { &ctx.enable } else { &ctx.disable };
    if let Err(error) = ctx.dev.write(msg) {
        return afb_error!("m4-rpc-fail", "power({}):{}", enable, error);
    };

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct SetPwmData {
    dev: Rc<TiRpmsg>,
}
AfbVerbRegister!(SetPwmCtrl, setpwm_callback, SetPwmData);
fn setpwm_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut SetPwmData,
) -> Result<(), AfbError> {
    let query = args.get::<JsoncObj>(0)?;

    let state = match query.get::<String>("action")?.to_uppercase().as_str() {
        "ON" => PwmState::On,
        "OFF" => PwmState::Off,
        "FAIL" => PwmState::F,
        _ => return afb_error!("setpwm-invalid-query", "action should be ON|OFF|FAIL"),
    };

    let duty = match query.get::<f64>("duty") {
        Ok(value) => value as f32,
        Err(_) => 0.0,
    };

    // this message cannot be build statically
    let msg = mk_pwm(&state, duty)?;
    if let Err(error) = ctx.dev.write(&msg) {
        return afb_error!("m4-rpc-fail", "set_pwm({:?}):{}", state, error);
    };
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct SetImaxData {
    dev: Rc<TiRpmsg>,
}
AfbVerbRegister!(SetImaxCtrl, set_imax_callback, SetImaxData);
fn set_imax_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut SetImaxData,
) -> Result<(), AfbError> {
    let imax = args.get::<u32>(0)?;
    let duty= imax as f32/ 60.0;

    // this message cannot be build statically
    let msg = mk_pwm(&PwmState::On, duty)?;
    if let Err(error) = ctx.dev.write(&msg) {
        return afb_error!("m4-rpc-fail", "set_imax({}) {}", imax, error);
    };
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct SetSlacData {
    dev: Rc<TiRpmsg>,
}
AfbVerbRegister!(SetSlacCtrl, setslac_callback, SetSlacData);
fn setslac_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut SetSlacData,
) -> Result<(), AfbError> {
    let status = args.get::<&SlacStatus>(0)?;

    let state = match status {
        SlacStatus::MATCHING => SlacState::Run,
        SlacStatus::MATCHED => SlacState::Ok,
        SlacStatus::TIMEOUT => SlacState::Nok,
        SlacStatus::UNMATCHED => SlacState::Nok,
        _ => SlacState::Udf,
    };

    // this message cannot be build statically
    let msg = mk_slac(&state)?;
    if let Err(error) = ctx.dev.write(&msg) {
        return afb_error!("m4-rpc-fail", "set_pwm({:?}):{}", state, error);
    };
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

pub(crate) fn register(
    rootv4: AfbApiV4,
    api: &mut AfbApi,
    config: &ApiUserData,
) -> Result<(), AfbError> {
    let ti_dev = TiRpmsg::new(config.cdev, config.rport, config.eptname)?;
    let handle = Rc::new(ti_dev);

    // force PWM off when starting
    let msg = mk_pwm(&PwmState::Off, 0.0)?;
    handle.write(&msg)?;

    // create event and store it within callback context
    let event = AfbEvent::new("iec");

    // register dev handler within listening event loop
    AfbEvtFd::new(config.uid)
        .set_fd(handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(Box::new(DevAsyncCtx {
            apiv4: rootv4,
            evt: event,
            count: Cell::new(0),
            dev: handle.clone(),
            lock_api: config.lock_api,
            lock_verb: config.lock_verb,
            imax: 0,
        }))
        .start()?;

    // set heartbeat timer
    AfbTimer::new(config.uid)
        .set_period(config.tic)
        .set_decount(0)
        .set_callback(Box::new(DevTimerCtx {
            heartbeat: mk_heartbeat()?,
            dev: handle.clone(),
        }))
        .start()?;

    let subscribe = AfbVerb::new("subscribe")
        .set_callback(Box::new(SubscribeCtrl { evt: event }))
        .set_info("subscribe Iec6185 event")
        .set_usage("true|false")
        .finalize()?;

    let ctx = EnableCtrl {
        dev: handle.clone(),
        enable: mk_enable()?,
        disable: mk_disable()?,
    };

    let dev_enable = AfbVerb::new("iec6185")
        .set_callback(Box::new(ctx))
        .set_info("enable/disable Iec6185 event (true/false)")
        .set_usage("true|false")
        .finalize()?;

    let ctx = SetPwmCtrl {
        dev: handle.clone(),
    };

    let set_pwm = AfbVerb::new("pwm")
        .set_callback(Box::new(ctx))
        .set_info("set_pwm")
        .set_usage("'action':'on/off','duty':0.05")
        .set_action("['on','off']")?
        .set_sample("{'action':'on', 'duty':0.05}")?
        .finalize()?;

    let ctx = SetImaxCtrl {
        dev: handle.clone(),
    };

    let set_imax = AfbVerb::new("imax")
        .set_callback(Box::new(ctx))
        .set_info("set_pwm")
        .set_usage("imax")
        .finalize()?;

    let ctx = SetSlacCtrl {
        dev: handle.clone(),
    };
    let slac_status = AfbVerb::new("slac")
        .set_callback(Box::new(ctx))
        .set_info("set slac status")
        .set_usage("SlacStatus Enum")
        .set_sample("'TIMEOUT'")?
        .set_sample("'MATCHED'")?
        .finalize()?;

    let ctx = PowerCtrl {
        dev: handle.clone(),
        enable: mk_power(true)?,
        disable: mk_power(false)?,
    };
    let allow_power = AfbVerb::new("power")
        .set_callback(Box::new(ctx))
        .set_info("allow power (true/false)")
        .set_usage("true/false")
        .finalize()?;

    api.add_event(event);
    api.add_verb(subscribe);
    api.add_verb(set_pwm);
    api.add_verb(set_imax);
    api.add_verb(dev_enable);
    api.add_verb(allow_power);
    api.add_verb(slac_status);

    // init m4 firmware (set pwm-off and enable iec6185 event)
    for msg in [mk_pwm(&PwmState::Off, 0.0)?, mk_enable()?] {
        if let Err(error) = handle.write(&msg) {
            return afb_error!("m4-init-fail", "firmware refused command error={}", error);
        }
    }

    Ok(())
}
