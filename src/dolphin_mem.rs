use std::ffi::c_void;
use std::mem;
use std::str::from_utf8_unchecked;

use anyhow::{anyhow, Result};
use neon::prelude::Context;
use neon::prelude::FunctionContext;
use neon::result::JsResult;
use neon::types::Finalize;
use neon::types::JsBox;
use neon::types::JsNumber;
use num_traits::FromPrimitive;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Memory::VirtualQueryEx;
use windows::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
use windows::Win32::System::ProcessStatus::QueryWorkingSetEx;
use windows::Win32::System::ProcessStatus::PSAPI_WORKING_SET_EX_BLOCK;
use windows::Win32::System::ProcessStatus::PSAPI_WORKING_SET_EX_INFORMATION;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, STILL_ACTIVE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
        },
        Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

extern crate num;

const VALID_PROCESS_NAMES: &'static [&'static str] = &[
    "Dolphin.exe",
    "Slippi Dolphin.exe",
    "DolphinWx.exe",
    "DolphinQt2.exe",
    "Citrus Dolphin.exe",
];
const GC_RAM_START: u32 = 0x80000000;
const GC_RAM_END: u32 = 0x81800000;
const GC_RAM_SIZE: usize = 0x2000000;
const MEM_MAPPED: u32 = 0x40000;

#[derive(FromPrimitive)]
enum ByteSize {
    U8 = 8,
    U16 = 16,
    U32 = 32,
}

struct DolphinMemoryFinderWindows;

impl DolphinMemoryFinderWindows {
    fn find_process_handle() -> Option<HANDLE> {
        unsafe {
            let mut status: u32 = 0;
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .expect("cannot create toolhelp32 snapshot");
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
                szExeFile: [0; 260],
            };

            let process_handle = loop {
                if !Process32Next(snapshot, &mut pe32 as *mut _).as_bool() {
                    break None;
                }

                let name = from_utf8_unchecked(&pe32.szExeFile);

                if !VALID_PROCESS_NAMES
                    .iter()
                    .any(|&valid_process_name| name.starts_with(valid_process_name))
                {
                    continue;
                };

                let handle = match OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    pe32.th32ProcessID,
                ) {
                    Ok(handle) => handle,
                    Err(_) => break None,
                };

                if GetExitCodeProcess(handle, &mut status as *mut _).as_bool()
                    && status as i32 == STILL_ACTIVE.0
                {
                    break Some(handle);
                }
            };
            CloseHandle(snapshot);
            return process_handle;
        }
    }

    fn find_gamecube_ram_offset(process_handle: HANDLE) -> Option<(usize, usize)> {
        unsafe {
            let mut info: MEMORY_BASIC_INFORMATION = Default::default();
            let mut address: usize = 0;

            while VirtualQueryEx(
                process_handle,
                Some(address as *const c_void),
                &mut info as *mut _,
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            ) == mem::size_of::<MEMORY_BASIC_INFORMATION>()
            {
                address = address + info.RegionSize / mem::size_of::<usize>();
                // Dolphin stores the GameCube RAM address space in 32MB chunks.
                // Extended memory override can allow up to 64MB.
                if !(info.RegionSize >= GC_RAM_SIZE
                    && info.RegionSize % GC_RAM_SIZE == 0
                    && info.Type.0 == MEM_MAPPED)
                {
                    continue;
                }

                let mut process_info = PSAPI_WORKING_SET_EX_INFORMATION {
                    VirtualAddress: 0 as *mut c_void,
                    VirtualAttributes: PSAPI_WORKING_SET_EX_BLOCK { Flags: 0 },
                };
                process_info.VirtualAddress = info.BaseAddress;

                if !QueryWorkingSetEx(
                    process_handle,
                    &mut process_info as *mut _ as *mut c_void,
                    mem::size_of::<PSAPI_WORKING_SET_EX_INFORMATION>()
                        .try_into()
                        .expect("cannot read process info"),
                )
                .as_bool()
                {
                    continue;
                }

                if (process_info.VirtualAttributes.Flags & 1) == 1
                    && info.BaseAddress != 0 as *mut c_void
                {
                    // Return as an object
                    return Some((info.BaseAddress as usize, info.RegionSize));
                }
            }
        }
        return None;
    }
}

#[derive(Copy, Clone)]
struct DolphinMemory {
    process_handle: HANDLE,
    dolphin_base_address: usize,
    dolphin_address_size: usize,
}

impl DolphinMemory {
    pub fn new(
        process_handle: HANDLE,
        dolphin_base_address: usize,
        dolphin_address_size: usize,
    ) -> Self {
        DolphinMemory {
            process_handle,
            dolphin_base_address,
            dolphin_address_size,
        }
    }

    pub fn init_with_process() -> Result<Self> {
        let process_handle = DolphinMemoryFinderWindows::find_process_handle()
            .ok_or(anyhow!("cannot find dolphin process"))?;

        let (dolphin_base_address, dolphin_address_size) =
            DolphinMemoryFinderWindows::find_gamecube_ram_offset(process_handle)
                .ok_or(anyhow!("cannot find gamecube ram offset"))?;

        Ok(Self::new(
            process_handle,
            dolphin_base_address,
            dolphin_address_size,
        ))
    }

    pub fn read<T: Sized>(self, addr: u32) -> Option<T>
    where
        [u8; mem::size_of::<T>()]:,
    {
        let mut addr = addr;
        if addr >= GC_RAM_START && addr <= GC_RAM_END {
            addr = addr % GC_RAM_START;
        } else {
            println!(
                "[MEMORY] Attempt to read from invalid address {:#08x}",
                addr
            );
            return None;
        }

        let raddr = self.dolphin_base_address as u32 + addr;
        let mut output = [0u8; mem::size_of::<T>()];
        let size = mem::size_of::<T>();
        let mut memread: usize = 0;

        unsafe {
            let success = ReadProcessMemory(
                self.process_handle,
                raddr as *const c_void,
                &mut output as *mut _ as *mut c_void,
                size,
                Some(&mut memread as *mut _),
            );
            if success.as_bool() && memread == size {
                output.reverse(); // reverse, as windows and dolphin has different endianness
                return Some(mem::transmute_copy(&output));
            } else {
                let err = GetLastError().0;
                println!(
                    "[MEMORY] Failed reading from address {:#08X} ERROR {}",
                    addr, err
                );

                return None;
            }
        }
    }
}

pub struct DolphinMemoryJs {
    dolphin_memory: DolphinMemory,
}

impl From<DolphinMemory> for DolphinMemoryJs {
    fn from(dolphin_memory: DolphinMemory) -> Self {
        Self::new(dolphin_memory)
    }
}

impl Finalize for DolphinMemoryJs {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {
        unsafe { CloseHandle(self.dolphin_memory.process_handle) };
    }
}

impl DolphinMemoryJs {
    fn new(dolphin_memory: DolphinMemory) -> Self {
        DolphinMemoryJs { dolphin_memory }
    }

    pub fn init_with_process(mut cx: FunctionContext) -> JsResult<JsBox<Self>> {
        match DolphinMemory::init_with_process() {
            Ok(dolphin_memory) => Ok(cx.boxed(DolphinMemoryJs::from(dolphin_memory))),
            Err(error) => cx.throw_error(error.to_string()),
        }
    }

    pub fn read(mut cx: FunctionContext) -> JsResult<JsNumber> {
        let address_js = cx.argument::<JsNumber>(0)?.value(&mut cx);
        let byte_size_js = cx.argument::<JsNumber>(1)?.value(&mut cx);

        let address = match u32::from_f64(address_js) {
            Some(address) => address,
            None => return cx.throw_error("invalid address"),
        };
        let byte_size = match ByteSize::from_f64(byte_size_js) {
            Some(address) => address,
            None => return cx.throw_error("invalid byte size"),
        };

        let memory = cx
            .this()
            .downcast_or_throw::<JsBox<DolphinMemory>, _>(&mut cx)?;

        const READ_ERROR_MESSAGE: &str = "failed reading from address";
        let memory_value = match byte_size {
            ByteSize::U8 => {
                let value = memory.read::<u8>(address);
                match value {
                    Some(value_js) => cx.number(value_js),
                    None => return cx.throw_error(READ_ERROR_MESSAGE),
                }
            }
            ByteSize::U16 => {
                let value = memory.read::<u16>(address);
                match value {
                    Some(value_js) => cx.number(value_js),
                    None => return cx.throw_error(READ_ERROR_MESSAGE),
                }
            }
            ByteSize::U32 => {
                let value = memory.read::<u32>(address);
                match value {
                    Some(value_js) => cx.number(value_js),
                    None => return cx.throw_error(READ_ERROR_MESSAGE),
                }
            }
        };

        Ok(memory_value)
    }
}
