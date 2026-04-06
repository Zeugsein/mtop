use crate::metrics::ProcessInfo;
use std::collections::HashMap;
use std::time::Instant;

/// Per-PID state for delta-based CPU% calculation.
/// Stores previous cumulative CPU time (Mach absolute units) and wall-clock timestamp.
pub struct ProcessCpuState {
    prev: HashMap<i32, (u64, Instant)>,
    timebase_numer: u32,
    timebase_denom: u32,
}

impl ProcessCpuState {
    pub fn new() -> Self {
        let (numer, denom) = mach_timebase();
        Self {
            prev: HashMap::new(),
            timebase_numer: numer,
            timebase_denom: denom,
        }
    }

    /// Convert Mach absolute time units to nanoseconds.
    fn mach_to_ns(&self, mach_time: u64) -> u64 {
        // On Apple Silicon numer/denom = 1/1, but handle Intel correctly
        mach_time * self.timebase_numer as u64 / self.timebase_denom as u64
    }
}

impl Default for ProcessCpuState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn collect_processes(cpu_state: &mut ProcessCpuState) -> Vec<ProcessInfo> {
    let pids = list_all_pids();
    let now = Instant::now();
    let mut procs = Vec::with_capacity(pids.len());
    let mut seen_pids = HashMap::with_capacity(pids.len());

    for pid in pids {
        if pid <= 0 {
            continue;
        }
        if let Some(info) = get_process_info(pid, cpu_state, now) {
            seen_pids.insert(pid, true);
            procs.push(info);
        }
    }

    // Remove stale PIDs no longer in the process list
    cpu_state.prev.retain(|pid, _| seen_pids.contains_key(pid));

    // Sort by CPU% descending
    procs.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Keep top 50
    procs.truncate(50);
    procs
}

/// Enumerate all PIDs via proc_listallpids
fn list_all_pids() -> Vec<i32> {
    // First call with null buffer to get count
    let count = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }

    // Allocate with headroom
    let capacity = (count as usize) + (count as usize) / 5;
    let mut pids: Vec<i32> = vec![0i32; capacity];
    let buf_size = (capacity * std::mem::size_of::<i32>()) as i32;

    let actual = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut libc::c_void, buf_size) };
    if actual <= 0 {
        return Vec::new();
    }

    pids.truncate(actual as usize);
    pids
}

/// Get process info via proc_pidinfo (PROC_PIDTASKINFO)
fn get_process_info(pid: i32, cpu_state: &mut ProcessCpuState, now: Instant) -> Option<ProcessInfo> {
    let mut task_info: ProcTaskInfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<ProcTaskInfo>() as i32;

    let ret = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTASKINFO,
            0,
            &mut task_info as *mut _ as *mut libc::c_void,
            size,
        )
    };

    if ret <= 0 {
        return None;
    }

    // Get process name via proc_name
    let name = get_proc_name(pid);
    if name.is_empty() {
        return None;
    }

    // Get username from UID
    let user = get_proc_user(pid);
    if user.is_empty() {
        return None;
    }

    // Delta-based CPU%: compare cumulative task time against wall-clock elapsed
    let cur_mach_time = task_info.pti_total_user + task_info.pti_total_system;
    let cpu_pct = if let Some(&(prev_mach_time, prev_wall)) = cpu_state.prev.get(&pid) {
        let delta_task_ns = cpu_state.mach_to_ns(cur_mach_time.saturating_sub(prev_mach_time));
        let delta_wall_ns = now.duration_since(prev_wall).as_nanos() as u64;
        if delta_wall_ns > 0 {
            (delta_task_ns as f64 / delta_wall_ns as f64 * 100.0) as f32
        } else {
            0.0
        }
    } else {
        // First sample for this PID — report 0%
        0.0
    };

    // Store current values for next delta
    cpu_state.prev.insert(pid, (cur_mach_time, now));

    let mem_bytes = task_info.pti_resident_size;

    Some(ProcessInfo {
        pid,
        name,
        cpu_pct,
        mem_bytes,
        user,
    })
}

fn get_proc_name(pid: i32) -> String {
    let mut buf = [0u8; 256];
    let ret = unsafe { proc_name(pid, buf.as_mut_ptr() as *mut libc::c_void, 256) };
    if ret <= 0 {
        return String::new();
    }
    let len = ret as usize;
    String::from_utf8_lossy(&buf[..len]).to_string()
}

fn get_proc_user(pid: i32) -> String {
    // Try PROC_PIDT_SHORTBSDINFO first (works in VMs and sandboxed environments)
    let mut short_info: ProcBsdShortInfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<ProcBsdShortInfo>() as i32;

    let ret = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDT_SHORTBSDINFO,
            0,
            &mut short_info as *mut _ as *mut libc::c_void,
            size,
        )
    };

    if ret > 0 {
        return uid_to_username(short_info.pbsi_uid);
    }

    // Fallback to PROC_PIDTBSDINFO
    let mut bsd_info: ProcBsdInfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<ProcBsdInfo>() as i32;

    let ret = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTBSDINFO,
            0,
            &mut bsd_info as *mut _ as *mut libc::c_void,
            size,
        )
    };

    if ret > 0 {
        return uid_to_username(bsd_info.pbi_uid);
    }

    String::new()
}

fn uid_to_username(uid: u32) -> String {
    unsafe {
        let mut pwd: libc::passwd = std::mem::zeroed();
        let mut result: *mut libc::passwd = std::ptr::null_mut();
        let mut buf = vec![0u8; 1024];

        let ret = libc::getpwuid_r(
            uid,
            &mut pwd,
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            &mut result,
        );

        if ret != 0 || result.is_null() {
            return uid.to_string();
        }

        std::ffi::CStr::from_ptr(pwd.pw_name)
            .to_string_lossy()
            .to_string()
    }
}

// --- Constants ---
const PROC_PIDTASKINFO: i32 = 4;
const PROC_PIDTBSDINFO: i32 = 2;
const PROC_PIDT_SHORTBSDINFO: i32 = 13;

// --- FFI structs ---
// These are simplified repr(C) structs matching the macOS kernel layout.
// Only the fields we access are named; the rest is padding.

#[repr(C)]
struct ProcTaskInfo {
    pti_virtual_size: u64,
    pti_resident_size: u64,
    pti_total_user: u64,
    pti_total_system: u64,
    pti_threads_user: u64,
    pti_threads_system: u64,
    pti_policy: i32,
    pti_faults: i32,
    pti_pageins: i32,
    pti_cow_faults: i32,
    pti_messages_sent: i32,
    pti_messages_received: i32,
    pti_syscalls_mach: i32,
    pti_syscalls_unix: i32,
    pti_csw: i32,
    pti_threadnum: i32,
    pti_numrunning: i32,
    pti_priority: i32,
}

// proc_bsdshortinfo from <sys/proc_info.h> — used with PROC_PIDT_SHORTBSDINFO
// 64 bytes total, works in VMs and sandboxed environments
#[repr(C)]
struct ProcBsdShortInfo {
    pbsi_pid: u32,          // offset 0
    pbsi_ppid: u32,         // offset 4
    pbsi_pgid: u32,         // offset 8
    pbsi_status: u32,       // offset 12
    pbsi_comm: [u8; 16],    // offset 16 (MAXCOMLEN)
    pbsi_flags: u32,        // offset 32
    pbsi_uid: u32,          // offset 36
    pbsi_gid: u32,          // offset 40
    pbsi_ruid: u32,         // offset 44
    pbsi_rgid: u32,         // offset 48
    pbsi_svuid: u32,        // offset 52
    pbsi_svgid: u32,        // offset 56
    _rfu: u32,              // offset 60
}

// proc_bsdinfo from <sys/proc_info.h> — used with PROC_PIDTBSDINFO
#[repr(C)]
struct ProcBsdInfo {
    pbi_flags: u32,
    pbi_status: u32,
    pbi_xstatus: u32,
    pbi_pid: u32,
    pbi_ppid: u32,
    pbi_uid: u32,
    pbi_gid: u32,
    pbi_ruid: u32,
    pbi_rgid: u32,
    pbi_svuid: u32,
    pbi_svgid: u32,
    _rfu_1: u32,
    pbi_comm: [u8; 16],   // MAXCOMLEN
    pbi_name: [u8; 32],   // 2*MAXCOMLEN
    pbi_nfiles: u32,
    pbi_pgid: u32,
    pbi_pjobc: u32,
    pbi_e_tdev: u32,
    pbi_e_tpgid: u32,
    pbi_nice: i32,
    pbi_start_tvsec: u64,
    pbi_start_tvusec: u64,
}

fn mach_timebase() -> (u32, u32) {
    let mut info = MachTimebaseInfo { numer: 0, denom: 0 };
    unsafe { mach_timebase_info(&mut info) };
    // Fallback to 1/1 if the call returns zeros (shouldn't happen)
    let numer = if info.numer == 0 { 1 } else { info.numer };
    let denom = if info.denom == 0 { 1 } else { info.denom };
    (numer, denom)
}

#[repr(C)]
struct MachTimebaseInfo {
    numer: u32,
    denom: u32,
}

unsafe extern "C" {
    fn proc_listallpids(buffer: *mut libc::c_void, buffersize: i32) -> i32;

    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut libc::c_void,
        buffersize: i32,
    ) -> i32;

    fn proc_name(pid: i32, buffer: *mut libc::c_void, buffersize: u32) -> i32;

    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
}
