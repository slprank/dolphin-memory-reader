// TODO Sessions - each scene has a minor 0 which is the css. if you leave the major scene, the session ends, otherwise when not in-game we show when the session started
// ^ option name "Show overall game session when not in-game" 
// TODO HRC & BTT Records in discord
// TODO Ranked match score, button "Viw opponent ranked profile", show details in stage striking already (in discord rich presence, signalize that you are in stage striking as well)
// TODO clean up melee.rs, move structs/enums away in coherent bundles

// #![windows_subsystem = "windows"]
#![feature(generic_const_exprs)]

use neon::prelude::*;

extern crate serde;
extern crate serde_json;

mod dolphin_mem;


#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("memoryNew", dolphin_mem::DolphinMemory::js_new);
    /*
    cx.export_function("memoryRead", dolphin_mem::DolphinMemory::js_read)?;
    cx.export_function("memoryReadString", dolphin_mem::DolphinMemory::js_read_string)?;
    */
}