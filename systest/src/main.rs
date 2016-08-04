#![allow(bad_style)]

extern crate libsodium_sys;
extern crate libc;

use libc::*;
use libsodium_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
