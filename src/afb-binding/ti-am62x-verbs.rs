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
use std::sync::Arc;

use crate::prelude::*;
use afbv4::prelude::*;

// protobuf maximum buffer size
const PROTOBUF_MAX_CAPACITY: usize = 256;

// import serde/json converters
AfbDataConverter!(pwm_state_type, PwmState);

// timer ctx and callback
struct DevTimerCtx {
    dev: Arc<TiRpmsg>,
    heartbeat: Vec<u8>,
}
AfbTimerRegister!(TimerCtrl, timer_callback, DevTimerCtx);
fn timer_callback(timer: &AfbTimer, _decount: u32, ctx: &mut DevTimerCtx) {
    // send heartbeat message
    match ctx.dev.write(&ctx.heartbeat) {
        Err(error) => {
            afb_log_msg!(Critical, timer, "{}", error);
        }
        Ok(()) => {}
    };
}

// on event ctx and callback
struct DevAsyncCtx {
    apiv4: AfbApiV4,
    count: Cell<u32>,
    dev: Arc<TiRpmsg>,
    evt: &'static AfbEvent,
}
AfbEvtFdRegister!(DecAsyncCtrl, async_dev_cb, DevAsyncCtx);
fn async_dev_cb(_evtfd: &AfbEvtFd, revent: u32, ctx: &mut DevAsyncCtx) {
    if revent == AfbEvtFdPoll::IN.bits() {
        let mut buffer: Vec<u8> = Vec::with_capacity(PROTOBUF_MAX_CAPACITY);
        match ctx.dev.read(&mut buffer) {
            Ok(value) => value,
            Err(error) => {
                afb_log_msg!(Critical, ctx.apiv4, "{}", error);
                return;
            }
        }

        match msg_uncode(&buffer) {
            EventMsg::Err(error) => {
                afb_log_msg!(Critical, ctx.apiv4, "{}", error);
            }

            EventMsg::Heartbeat() => {
                let count = ctx.count.get() + 1;
                ctx.count.set(count);

                afb_log_msg!(Debug, ctx.apiv4, "Device heartbeat count={}", count);
            }

            EventMsg::Msg(iso6185) => {
                ctx.evt.push(iso6185.as_str_name());
            }
        };
    }
}

struct SubscribeData {
    evt: &'static AfbEvent,
}
AfbVerbRegister!(SubscribeCtrl, subscribe_callback, SubscribeData);
fn subscribe_callback(
    request: &AfbRequest,
    _args: &AfbData,
    ctx: &mut SubscribeData,
) -> Result<(), AfbError> {
    ctx.evt.subscribe(request)?;
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct UnsubscribeData {
    evt: &'static AfbEvent,
}
AfbVerbRegister!(UnsubscribeCtrl, unsubscribe_callback, UnsubscribeData);
fn unsubscribe_callback(
    request: &AfbRequest,
    _args: &AfbData,
    ctx: &mut UnsubscribeData,
) -> Result<(), AfbError> {
    ctx.evt.unsubscribe(request)?;
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct EnableData {
    dev: Arc<TiRpmsg>,
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

    match ctx.dev.write(msg) {
        Err(error) => {
            afb_log_msg!(Critical, request, "enable({}):{}", enable, error);
        }
        Ok(()) => {}
    };
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct PowerData {
    dev: Arc<TiRpmsg>,
    enable: Vec<u8>,
    disable: Vec<u8>,
}
AfbVerbRegister!(PowerCtrl, power_callback, PowerData);
fn power_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut PowerData,
) -> Result<(), AfbError> {
    let power = args.get::<bool>(0)?;
    let msg = if power { &ctx.enable } else { &ctx.disable };

    match ctx.dev.write(msg) {
        Err(error) => {
            afb_log_msg!(Critical, request, "power(allow:{}):{}", power, error);
        }
        Ok(()) => {}
    };
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct SetpwmData {
    dev: Arc<TiRpmsg>,
}
AfbVerbRegister!(SetpwmCtrl, setpwm_callback, SetpwmData);
fn setpwm_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut SetpwmData,
) -> Result<(), AfbError> {
    let state = args.get::<&PwmState>(0)?;
    let cycle = args.get::<f64>(1)?;

    // this message cannot be build statically
    let msg = mk_pwm(state, cycle as f32)?;

    match ctx.dev.write(&msg) {
        Err(error) => {
            afb_log_msg!(Critical, request, "setpwm:{}", error);
        }
        Ok(()) => {}
    };
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

pub(crate) fn register(api: &mut AfbApi, config: &ApiUserData) -> Result<(), AfbError> {
    let ti_dev = TiRpmsg::new(config.devname, config.eptnum, config.eptname)?;
    let handle = Arc::new(ti_dev);

    // create event and store it within callback context
    let event = AfbEvent::new(config.uid);

    // register imported serde type
    pwm_state_type::register()?;

    // register dev handler within listening event loop
    AfbEvtFd::new(config.uid)
        .set_fd(handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(Box::new(DevAsyncCtx {
            apiv4: api.get_apiv4(),
            dev: handle.clone(),
            evt: event,
            count: Cell::new(0),
        }))
        .start()?;

    // set heartbeat timer
    match AfbTimer::new(config.uid)
        .set_period(config.tic)
        .set_decount(0)
        .set_callback(Box::new(DevTimerCtx {
            heartbeat: mk_heartbeat()?,
            dev: handle.clone(),
        }))
        .start()
    {
        Err(error) => {
            afb_log_msg!(Critical, api.get_apiv4(), &error);
            Err(error)
        }
        Ok(_timer) => Ok(()),
    }?;

    let unsubscribe = AfbVerb::new("unsubscribe")
        .set_callback(Box::new(UnsubscribeCtrl { evt: event }))
        .set_info("unsubscribe Iec6185 event")
        .set_usage("no input")
        .finalize()?;

    let subscribe = AfbVerb::new("subscribe")
        .set_callback(Box::new(SubscribeCtrl { evt: event }))
        .set_info("unsubscribe Iec6185 event")
        .set_usage("no input")
        .finalize()?;

    let dev_enable = AfbVerb::new("enable")
        .set_callback(Box::new(EnableCtrl {
            dev: handle.clone(),
            enable: mk_enable()?,
            disable: mk_disable()?,
        }))
        .set_info("enable/disable Iec6185 event")
        .set_usage("true/false")
        .finalize()?;

    let allow_power = AfbVerb::new("power")
        .set_callback(Box::new(PowerCtrl {
            dev: handle.clone(),
            enable: mk_power(true)?,
            disable: mk_power(false)?,
        }))
        .set_info("power/disable Iec6185 event")
        .set_usage("true/false")
        .finalize()?;

    api.add_event(event);
    api.add_verb(subscribe);
    api.add_verb(unsubscribe);
    api.add_verb(dev_enable);
    api.add_verb(allow_power);

    Ok(())
}
