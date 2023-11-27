/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * $RP_BEGIN_LICENSE$
 * Commercial License Usage
 *  Licensees holding valid commercial IoT.bzh licenses may use this file in
 *  accordance with the commercial license agreement provided with the
 *  Software or, alternatively, in accordance with the terms contained in
 *  a written agreement between you and The IoT.bzh Company. For licensing terms
 *  and conditions see https://www.iot.bzh/terms-conditions. For further
 *  information use the contact form at https://www.iot.bzh/contact.
 *
 * GNU General Public License Usage
 *  Alternatively, this file may be used under the terms of the GNU General
 *  Public license version 3. This license is as published by the Free Software
 *  Foundation and appearing in the file LICENSE.GPLv3 included in the packaging
 *  of this file. Please review the following information to ensure the GNU
 *  General Public License requirements will be met
 *  https://www.gnu.org/licenses/gpl-3.0.html.
 *  $RP_END_LICENSE$
 */

#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod cglue {
    include!("./_capi-map.rs");
}

use afbv4::prelude::AfbError;
//use std::ffi::CStr;
use std::ffi::CString;

pub struct TiRpmsg {
    pub(self) handle: *mut cglue::rpmsg_char_dev,
}

// This function initialize ti-rmsg lib. Use socname=Null for auto detection
// Return nothing or error. Note that this function should be called only once
// at library initialization anf before any other rpmsg call.
pub fn ti_init(socname: Option<&str>) -> Result<(), AfbError> {
    let name = match socname {
        None => 0 as *mut ::std::os::raw::c_char,
        Some(value) => {
            let sname = CString::new(value).expect("Invalid name string");
            sname.into_raw()
        }
    };

    let rc = unsafe { cglue::rpmsg_char_init(name) };
    if rc < 0 {
        return Err(AfbError::new("ti-rmsg-init", "Fail to initialize library"));
    }

    Ok(())
}

pub fn ti_exit() {
    unsafe { cglue::rpmsg_char_exit() };
}

impl TiRpmsg {
    pub fn new(cdev: Option<&str>, rport: i32, eptname: &str) -> Result<TiRpmsg, AfbError> {
        let cdev = match cdev {
            None => 0 as *mut ::std::os::raw::c_char,
            Some(value) => {
                let sname = CString::new(value).expect("Invalid cdev string");
                sname.into_raw()
            }
        };

        let eptname = CString::new(eptname).expect("Invalid eptname string");

        let handle = unsafe {
            cglue::rpmsg_char_open(
                cglue::rproc_id_M4F_MCU0_0,
                cdev,
                -1, /* any port */
                rport,
                eptname.into_raw(),
                0,
            )
        };

        if handle == 0 as *mut cglue::rpmsg_char_dev {
            return Err(AfbError::new(
                "ti-rmsg-open",
                "Fail to open ti-rpmsg device",
            ));
        }

        Ok(TiRpmsg { handle })
    }

    pub fn get_fd(&self) -> ::std::os::raw::c_int {
        let handle = unsafe { &mut *self.handle };
        handle.fd
    }

    pub fn write(&self, buffer: &Vec<u8>) -> Result<(), AfbError> {
        // extract raw buffer from vector
        let len = buffer.capacity();
        let ptr = buffer.as_ptr() as *mut ::std::os::raw::c_void;

        // extract C mutable handle and write buffer
        let handle = unsafe { &mut *self.handle };
        let count = unsafe { cglue::write(handle.fd, ptr, len) };
        if count != len as isize {
            return Err(AfbError::new(
                "rpmsg-write-fail",
                format!("fail to write bytes:{} count:{}", len, count),
            ));
        }
        Ok(())
    }

    pub fn read(&self, buffer: &mut[u8]) -> Result<usize, AfbError> {

        // extract C mutable handle and write buffer
        let handle = unsafe { &mut *self.handle };

        // extract raw buffer from vector
        let len = buffer.len();
        let ptr = buffer.as_mut_ptr() as *mut ::std::os::raw::c_void;

        let count = unsafe { cglue::read(handle.fd, ptr, len) };
        if count == len as isize {
            return Err(AfbError::new(
                "rpmsg-read-fail",
                format!("fail to read (buffer too-small?) count={}", count),
            ));
        }
        Ok(count as usize)
    }
}
