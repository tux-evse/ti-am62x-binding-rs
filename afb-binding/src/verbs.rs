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

// protobuf maximum buffer size
const PROTOBUF_MAX_CAPACITY: usize = 256;

// timer ctx and callback
struct DevTimerCtx {
    dev: Rc<TiRpmsg>,
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

// FCAM: Jobpost used to manage the delay after Lock/Unlock command
struct UserPostData {
    event: AfbEvent,
    apiv4: AfbApiv4,
    api_gpio: &'static str,
    lock: bool,
}
// this callback starts from AfbSchedJob::new. If signal!=0 then callback overpass its watchdog timeout
AfbJobRegister!(DelayCtrl, jobpost_callback, UserPostData);
fn jobpost_callback(job: &AfbSchedJob, signal: i32, userdata: &mut UserPostData) {
    afb_log_msg!(
        Info,
        job,
        "{}: jobpost callback Lock-Unlock_signal={}",
        job.get_uid(),
        signal
    );
    match AfbSubCall::call_sync(userdata.apiv4, userdata.api_gpio, "lock", userdata.lock) {
        Err(error)=> { 
            afb_log_msg!(
                Error,
                job,
                " jobpost callsync api: {}/lock fail",
                userdata.api_gpio,
            );
        },
        Ok(args)=> {
            let json=JsoncObj::new(); 
            json.add("action", "lock");
            json.add("status", userdata.lock);
            userdata.event.push(json);

            //
            let msg=mk_power(userdata.lock).unwrap();
            ctx.dev.write(msg).unwrap();

        },
}

// post a job at 100ms with a clone of the received json query
struct UserPostVerb {
    event: &'static AfbEvent,
}

fn jobpost(context: UserPostData) {
    if let Err(error) = AfbSchedJob::new("Jobpost Lock/Unlock ")
        .set_exec_watchdog(2) // limit exec time to 200ms;
        .set_callback(Box::new(context))
        .post(100)
    {
        afb_log_msg!(
            Info,
            None,
            "Lock/Unlock Job Failed uid:{} jobid={}",
            job.get_uid(),
            job.get_jobid()
        );
    }
}

// on event ctx and callback
struct DevAsyncCtx {
    count: Cell<u32>,
    dev: Rc<TiRpmsg>,
    evt: &'static AfbEvent,
    api_gpio: &'static str,
    apiv4: AfbApiV4,
}

AfbEvtFdRegister!(DecAsyncCtrl, async_dev_cb, DevAsyncCtx);
fn async_dev_cb(_event: &AfbEvtFd, revent: u32, ctx: &mut DevAsyncCtx) {
    if revent == AfbEvtFdPoll::IN.bits() {
        #[allow(invalid_value)]
        let mut buffer: [u8; PROTOBUF_MAX_CAPACITY as usize] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let len = match ctx.dev.read(&mut buffer) {
            Ok(len) => len,
            Err(error) => {
                afb_log_msg!(Critical, None, "{}", error);
                return;
            }
        };

        let data = &buffer[0..len];
        afb_log_msg!(Debug, None, format!("rpmsg data={:X?}(hexa)", data));

        match msg_uncode(data) {
            EventMsg::Err(error) => {
                afb_log_msg!(Critical, None, "{}", error);
            }

            EventMsg::Heartbeat() => {
                let count = ctx.count.get() + 1;
                ctx.count.set(count);
                afb_log_msg!(Debug, None, "Device heartbeat count={}", count);
            }

            EventMsg::Msg(iso6185) => {
                ctx.evt.push(iso6185.as_str_name());

                match iso6185 {
                    CarPluggedIn => {
                        // Lock motor on CarPlugging
                        // Call I2C API to lock (callsync)
                        let context = UserPostData {
                            event: ctx.evt,
                            apiv4: ctx.apiv4,
                            api_gpio: ctx.api_gpio, // "i2c/gpio",
                            lock: true,
                        };
                        jobpost(context);

                        // Delay 100ms jobpost

                        // Read the lock status

                        // Event/Rpmsg Lock OK/NOK
                    }
                    CarRequestedStopPower => {
                        // Unlock motor on StopPower request
                        // Call I2C API to lock
                        let context = UserPostData {
                            event: ctx.evt,
                            apiv4: ctx.apiv4,
                            api_gpio: ctx.api_gpio, // "i2c/gpio",
                            lock: false,
                        };
                        jobpost(context);

                        // Delay 100ms jobpost

                        // Read the Unlock status

                        // Event/Rpmsg Unlock OK/NOK
                    }
                    _ => {} // others do nothing
                }
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
    dev: Rc<TiRpmsg>,
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
    // register custom afb-v4 type converter
    rpmsg_register()?;

    let ti_dev = TiRpmsg::new(config.cdev, config.rport, config.eptname)?;
    let handle = Rc::new(ti_dev);

    // create event and store it within callback context
    let event = AfbEvent::new(config.uid);

    // register dev handler within listening event loop
    AfbEvtFd::new(config.uid)
        .set_fd(handle.get_fd())
        .set_events(AfbEvtFdPoll::IN)
        .set_callback(Box::new(DevAsyncCtx {
            dev: handle.clone(),
            evt: event,
            count: Cell::new(0),
            apiv4: api.get_apiv4(),
            api_gpio: config.api_gpio,
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

    let ctx = EnableCtrl {
        dev: handle.clone(),
        enable: mk_enable()?,
        disable: mk_disable()?,
    };
    let dev_enable = AfbVerb::new("enable")
        .set_callback(Box::new(ctx))
        .set_info("enable/disable Iec6185 event")
        .set_usage("true/false")
        .finalize()?;

    let ctx = PowerCtrl {
        dev: handle.clone(),
        enable: mk_power(true)?,
        disable: mk_power(false)?,
    };
    let allow_power = AfbVerb::new("power")
        .set_callback(Box::new(ctx))
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
