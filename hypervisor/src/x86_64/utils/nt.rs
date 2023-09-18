#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

use alloc::vec::Vec;
use winapi::{
    km::wdm::KIRQL,
    shared::ntdef::{
        NTSTATUS, PGROUP_AFFINITY, PHYSICAL_ADDRESS, PPROCESSOR_NUMBER, PVOID, UNICODE_STRING,
    },
    um::winnt::CONTEXT,
};

extern "system" {
    pub static KdDebuggerNotPresent: *mut bool;
}

#[link(name = "ntoskrnl")]
extern "system" {
    pub fn KeGetCurrentIrql() -> KIRQL;

    //This wont work as the function is not in ntoskrnl.lib or hal.lib so we use MmGetSystemRoutineAddress to get the address
    //pub fn KeRaiseIrqlToDpcLevel() -> KIRQL;

    pub fn KfRaiseIrql(new_irql: KIRQL) -> KIRQL;

    pub fn KeLowerIrql(new_irql: KIRQL);

    pub fn MmGetSystemRoutineAddress(system_routine_name: *mut UNICODE_STRING) -> PVOID;

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-mmgetphysicaladdress
    pub fn MmGetPhysicalAddress(BaseAddress: PVOID) -> PHYSICAL_ADDRESS;

    ///undocumented
    pub fn MmGetVirtualForPhysical(PhysicalAddress: PHYSICAL_ADDRESS) -> *mut u64;

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-kequeryactiveprocessorcountex
    pub fn KeQueryActiveProcessorCountEx(GroupNumber: u16) -> u32;

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-kegetcurrentprocessornumberex
    pub fn KeGetCurrentProcessorNumberEx(ProcNumber: *mut u64) -> u32;

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-kegetprocessornumberfromindex
    pub fn KeGetProcessorNumberFromIndex(ProcIndex: u32, ProcNumber: PPROCESSOR_NUMBER)
        -> NTSTATUS;

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-kesetsystemgroupaffinitythread
    pub fn KeSetSystemGroupAffinityThread(
        Affinity: PGROUP_AFFINITY,
        PreviousAffinity: PGROUP_AFFINITY,
    );

    ///undocumented
    pub fn ZwYieldExecution() -> NTSTATUS;

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-kereverttousergroupaffinitythread
    pub fn KeRevertToUserGroupAffinityThread(PreviousAffinity: PGROUP_AFFINITY);

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-rtlinitializebitmap
    pub fn RtlInitializeBitMap(
        BitMapHeader: PRTL_BITMAP,
        BitMapBuffer: *mut u32,
        SizeOfBitMap: u32,
    );

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-rtlclearallbits
    pub fn RtlClearAllBits(BitMapHeader: PRTL_BITMAP);

    ///https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtlcapturecontext
    pub fn RtlCaptureContext(ContextRecord: *mut Context);

    ///https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nf-ntddk-kebugcheck
    pub fn KeBugCheck(BugCheckCode: u32) -> !;
}

// There is a bug in windows-rs/windows-sys and WINAPI: https://github.com/microsoft/win32metadata/issues/1044. Otherwise this is not needed.
#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct Context(pub CONTEXT);

impl core::ops::Deref for Context {
    type Target = CONTEXT;
    fn deref(&self) -> &CONTEXT {
        &self.0
    }
}

impl core::ops::DerefMut for Context {
    fn deref_mut(&mut self) -> &mut CONTEXT {
        &mut self.0
    }
}

// See: https://docs.microsoft.com/en-us/windows-hardware/drivers/debugger/bug-check-code-reference2#bug-check-codes
pub const MANUALLY_INITIATED_CRASH: u32 = 0x000000E2;

/// Passive release level
pub const PASSIVE_LEVEL: KIRQL = 0;
/// Lowest interrupt level
pub const LOW_LEVEL: KIRQL = 0;
/// APC interrupt level
pub const APC_LEVEL: KIRQL = 1;
/// Dispatcher level
pub const DISPATCH_LEVEL: KIRQL = 2;
/// CMCI interrupt level
pub const CMCI_LEVEL: KIRQL = 5;

/// Interval clock level
pub const CLOCK_LEVEL: KIRQL = 13;
/// Interprocessor interrupt level
pub const IPI_LEVEL: KIRQL = 14;
/// Deferred Recovery Service level
pub const DRS_LEVEL: KIRQL = 14;
/// Power failure level
pub const POWER_LEVEL: KIRQL = 14;
/// Timer used for profiling.
pub const PROFILING_LEVEL: KIRQL = 15;
/// Highest interrupt level
pub const HIGH_LEVEL: KIRQL = 15;

#[repr(C)]
pub struct RTL_BITMAP {
    pub(crate) SizeOfBitMap: u32,
    pub(crate) Buffer: *mut u32,
}

pub type PRTL_BITMAP = *mut RTL_BITMAP;

/// Gets ta pointer to a function from ntoskrnl exports
fn get_ntoskrnl_export(function_name: *mut UNICODE_STRING) -> PVOID {
    // The MmGetSystemRoutineAddress routine returns a pointer to a function specified by SystemRoutineName.
    unsafe { MmGetSystemRoutineAddress(function_name) }
}

/// Raises the current IRQL to DISPATCH_LEVEL and returns the previous IRQL.
pub fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    type FnKeRaiseIrqlToDpcLevel = unsafe extern "system" fn() -> KIRQL;

    // Include the null terminator for the C-style API
    let wide_string: Vec<u16> = "KeRaiseIrqlToDpcLevel\0".encode_utf16().collect();

    let mut unicode_string = UNICODE_STRING {
        // Length in bytes, excluding the null terminator
        Length: ((wide_string.len() - 1) * 2) as u16,
        MaximumLength: (wide_string.len() * 2) as u16,
        Buffer: wide_string.as_ptr() as *mut _,
    };

    // Get the address of the function from ntoskrnl
    let routine_address = get_ntoskrnl_export(&mut unicode_string);

    let pKeRaiseIrqlToDpcLevel =
        unsafe { core::mem::transmute::<PVOID, FnKeRaiseIrqlToDpcLevel>(routine_address) };

    // Ensure the wide_string doesn't get dropped while the UNICODE_STRING is in use
    core::mem::forget(wide_string);

    // Invoke the retrieved function
    unsafe { pKeRaiseIrqlToDpcLevel() }
}

/// Lowers the current IRQL to the specified value.
pub fn KeLowerIrqlToOldLevel(old_irql: KIRQL) {
    unsafe { KeLowerIrql(old_irql) };
}
