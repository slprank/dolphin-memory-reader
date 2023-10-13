use std::ffi::c_void;
use std::mem;
use std::str::from_utf8_unchecked;

use encoding_rs::SHIFT_JIS;
use neon::prelude::Context;
use neon::prelude::FunctionContext;
use neon::result::JsResult;
use neon::types::Finalize;
use neon::types::JsBox;
use neon::types::JsNumber;
use num_traits::FromPrimitive;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
use windows::Win32::System::Memory::VirtualQueryEx;
use windows::Win32::System::ProcessStatus::PSAPI_WORKING_SET_EX_BLOCK;
use windows::Win32::System::ProcessStatus::PSAPI_WORKING_SET_EX_INFORMATION;
use windows::Win32::System::ProcessStatus::QueryWorkingSetEx;
use windows::Win32::{System::{Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, PROCESSENTRY32, TH32CS_SNAPPROCESS, Process32Next}, Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, GetExitCodeProcess}}, Foundation::{STILL_ACTIVE, HANDLE, CloseHandle}};

extern crate num;

const VALID_PROCESS_NAMES: &'static [&'static str] = &["Dolphin.exe", "Slippi Dolphin.exe", "DolphinWx.exe", "DolphinQt2.exe", "Citrus Dolphin.exe",];
const GC_RAM_START: u32 = 0x80000000;
const GC_RAM_END: u32 = 0x81800000;
const GC_RAM_SIZE: usize = 0x2000000;
const MEM_MAPPED: u32 = 0x40000;


#[derive(Copy, Clone)]
pub struct DolphinMemory {
    process_handle: Option<HANDLE>,
    dolphin_base_addr: Option<usize>,
    dolphin_addr_size: Option<usize>
}

pub fn init_memory_read() -> DolphinMemory {
    let mut process_handle: Option<HANDLE>  = None;
    let mut dolphin_base_address: Option<usize>  = None;
    let mut dolphin_address_size: Option<usize>  = None;
    loop {
        if process_handle.is_none() {
            process_handle = find_process()
        }
        if process_handle.is_some() && dolphin_base_address.is_none() || dolphin_address_size.is_none() {
            (dolphin_base_address, dolphin_address_size) = find_gamecube_ram_offset(process_handle)
        }

        if process_handle.is_some() && dolphin_base_address.is_some() && dolphin_base_address.is_some() {
            return DolphinMemory::new(process_handle, dolphin_base_address, dolphin_address_size)
        }
    }
}

fn find_process() -> Option<HANDLE> {
    unsafe {
        let mut status: u32 = 0;
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
        let mut pe32 = PROCESSENTRY32 {
            dwSize: mem::size_of::<PROCESSENTRY32>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; 260]
        };

        loop {
            if !Process32Next(snapshot, &mut pe32 as *mut _).as_bool() {
                break;
            }
            let name = from_utf8_unchecked(&pe32.szExeFile);
            if VALID_PROCESS_NAMES.iter().any(|&e| name.starts_with(e)) {
                println!("{}", name);
                let handle_res = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pe32.th32ProcessID);
                if handle_res.is_ok() {
                    let handle = handle_res.unwrap();
                    if GetExitCodeProcess(handle, &mut status as *mut _).as_bool() && status as i32 == STILL_ACTIVE.0 {
                        return Some(handle);
                    }
                } else {
                    // ? handle is supposed to be null so what will be closed... ported from m-overlay, see reference on the top
                    CloseHandle(handle_res.unwrap());
                    return None;
                }
            } else {
                return None;
            }
        }
        CloseHandle(snapshot);
        return None
    }
}

fn find_gamecube_ram_offset(process_handle: Option<HANDLE>) -> (Option<usize>, Option<usize>) {
    unsafe {
        let mut info: MEMORY_BASIC_INFORMATION = Default::default();
        let mut address: usize = 0;

        while VirtualQueryEx(process_handle.unwrap(), Some(address as *const c_void), &mut info as *mut _, mem::size_of::<MEMORY_BASIC_INFORMATION>()) == mem::size_of::<MEMORY_BASIC_INFORMATION>() {
            address = address + info.RegionSize / mem::size_of::<usize>();
            // Dolphin stores the GameCube RAM address space in 32MB chunks.
            // Extended memory override can allow up to 64MB.
            if info.RegionSize >= GC_RAM_SIZE && info.RegionSize % GC_RAM_SIZE == 0 && info.Type.0 == MEM_MAPPED {
                let mut wsinfo = PSAPI_WORKING_SET_EX_INFORMATION {
                    VirtualAddress: 0 as *mut c_void,
                    VirtualAttributes: PSAPI_WORKING_SET_EX_BLOCK { Flags: 0 }
                };
                wsinfo.VirtualAddress = info.BaseAddress;

                if QueryWorkingSetEx(process_handle.unwrap(), &mut wsinfo as *mut _ as *mut c_void, mem::size_of::<PSAPI_WORKING_SET_EX_INFORMATION>().try_into().unwrap()).as_bool() {
                    if (wsinfo.VirtualAttributes.Flags & 1) == 1 && info.BaseAddress != 0 as *mut c_void {
                        // Return as an object
                        return (Some(info.BaseAddress as usize), Some(info.RegionSize));
                    }
                }
            }
        }
    }
    return (None, None);
}

impl DolphinMemory {
    pub fn new(process_handle: Option<HANDLE>, dolphin_base_address: Option<usize>, dolphin_address_size: Option<usize>) -> Self {
        DolphinMemory { process_handle: process_handle, dolphin_base_addr: dolphin_base_address, dolphin_addr_size: dolphin_address_size }
    }

    pub fn read<T: Sized>(self, addr: u32) -> Option<T> where [u8; mem::size_of::<T>()]:{
        let mut addr = addr;
        if addr >= GC_RAM_START && addr <= GC_RAM_END {
            addr = addr % GC_RAM_START;
        } else {
            println!("[MEMORY] Attempt to read from invalid address {:#08x}", addr);
		    return None;
        }

        let raddr = self.dolphin_base_addr.unwrap() as u32 + addr;
        let mut output = [0u8; mem::size_of::<T>()];
        let size = mem::size_of::<T>();
        let mut memread: usize = 0;
        
        unsafe {
            let success = ReadProcessMemory(self.process_handle.unwrap(), raddr as *const c_void, &mut output as *mut _ as *mut c_void, size, Some(&mut memread as *mut _));
            if success.as_bool() && memread == size {
                // because win32 decides to give me the output in the wrong endianness, we'll reverse it
                output.reverse(); // TODO figure out if we really have to do this, i would like to avoid it if possible
                return Some(mem::transmute_copy(&output));
            } else {
                let err = GetLastError().0;
                println!("[MEMORY] Failed reading from address {:#08X} ERROR {}", addr, err);

                return None;
            }
        }
    }

    // pub fn read_string<const LEN: usize>(self, addr: u32) -> Option<String> where [(); mem::size_of::<[u8; LEN]>()]:{
    //     let res = self.read::<[u8; LEN]>(addr);
    //     if res.is_none() {
    //         return None;
    //     }

    //     let mut raw = res.unwrap();
    //     raw.reverse(); // we apparently have to reverse it again due to how the string is gathered

    //     return match std::str::from_utf8(&raw) {
    //         Ok(v) => Some(v.trim_end_matches(char::from(0)).into()),
    //         Err(e) => {
    //             println!("Invalid utf-8 string => {:?} | {}", res.unwrap(), e.to_string());
    //             None
    //         }
    //     };
    // }

    // pub fn read_string_shift_jis<const LEN: usize>(&mut self, addr: u32) -> Option<String> where [(); mem::size_of::<[u8; LEN]>()]:{
    //     let res = self.read::<[u8; LEN]>(addr);
    //     if res.is_none() {
    //         return None;
    //     }

    //     let mut raw = res.unwrap();
    //     raw.reverse(); // we apparently have to reverse it again due to how the string is gathered

    //     let (dec_res, _enc, errors) = SHIFT_JIS.decode(&raw);
    //     if errors {
    //         println!("Invalid shift-jis string => {:?}", res.unwrap())
    //     }
    //     return Some(dec_res.as_ref().trim_end_matches(char::from(0)).to_string());
    // }

    // pub fn pointer_indirection(&mut self, addr: u32, amount: u32) -> Option<u32> {
    //     let mut curr = self.read::<u32>(addr);
    //     for n in 2..=amount {
    //         if curr.is_none() {
    //             return None;
    //         }
    //         curr = self.read::<u32>(curr.unwrap());
    //     }
    //     curr
    // }    
}

#[derive(FromPrimitive)]
enum ByteSize {
   U8 = 8,
   U16 = 16,
   U32 = 32,
}

impl DolphinMemory {
    pub fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<DolphinMemory>> {
        let memory = init_memory_read();
        Ok(cx.boxed(memory))
    }
    
    pub fn js_read(mut cx: FunctionContext) -> JsResult<JsNumber> {
        let address_js = cx.argument::<JsNumber>(0)?.value(&mut cx);
        let byte_size_js = cx.argument::<JsNumber>(1)?.value(&mut cx);

        let address = u32::from_f64(address_js).unwrap();
        let byte_size = ByteSize::from_f64(byte_size_js).unwrap();

        let memory = cx.this().downcast_or_throw::<JsBox<DolphinMemory>, _>(&mut cx)?;

        let memory_value = match byte_size {
            ByteSize::U8 => {
                let value = memory.read::<u8>(address);
                match value {
                  Some(value_js) => cx.number(value_js),
                  None => {return cx.throw_error("rrRrorRRR")}
                }
            }
            ByteSize::U16 => {
                let value = memory.read::<u8>(address);
                match value {
                  Some(value_js) => cx.number(value_js),
                  None => {return cx.throw_error("rrRrorRRR")}
                }
            }
            ByteSize::U32 => {
                let value = memory.read::<u8>(address);
                match value {
                  Some(value_js) => cx.number(value_js),
                  None => {return cx.throw_error("rrRrorRRR")}
                }
            }
        };

        Ok(memory_value)
    }
    
    // pub fn js_read_string(mut cx: FunctionContext) -> JsResult<JsString> {
    //     let address_js = cx.argument::<JsNumber>(0)?.value(&mut cx);
    //     let chars_js= cx.argument::<JsNumber>(1)?.value(&mut cx);

    //     let address = u32::from_f64(address_js).unwrap();
    //     let chars = usize::from_f64(chars_js).unwrap();

    //     let memory = cx.this().downcast_or_throw::<JsBox<DolphinMemory>, _>(&mut cx)?;

    //     let memory_string = {
    //         let value = memory.read_string::<chars>(address);
    //             match value {
    //               Some(value_js) => cx.string(value_js),
    //               None => {return cx.throw_error("rrRrorRRR")}
    //             }
    //     };

    //     Ok(memory_string)
    // }
}

impl Finalize for DolphinMemory {}

pub mod util {
    macro_rules! R13 {($offset:expr) => { 0x804db6a0 - $offset }}
    pub(crate) use R13;
}
