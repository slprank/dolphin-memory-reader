// TODO Sessions - each scene has a minor 0 which is the css. if you leave the major scene, the session ends, otherwise when not in-game we show when the session started
// ^ option name "Show overall game session when not in-game" 
// TODO HRC & BTT Records in discord
// TODO Ranked match score, button "Viw opponent ranked profile", show details in stage striking already (in discord rich presence, signalize that you are in stage striking as well)
// TODO clean up melee.rs, move structs/enums away in coherent bundles

// #![windows_subsystem = "windows"]
#![feature(generic_const_exprs)]

extern crate serde;
extern crate serde_json;

mod util;
mod melee;

fn main() {
    // TODO: Provide PID, Base Address and RegionSize if possible
    let mut client = melee::MeleeClient::new();



    // Return Memory Value
}