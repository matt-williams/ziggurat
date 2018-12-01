#![allow(dead_code, unused_parens, unused_imports)]

#[macro_use]
extern crate stdweb as _stdweb;
#[macro_use]
extern crate serde_derive as _serde_derive;
#[macro_use]
extern crate stdweb_derive as _stdweb_derive;

include!(concat!(env!("OUT_DIR"), "/webgl_rendering_context.rs"));
