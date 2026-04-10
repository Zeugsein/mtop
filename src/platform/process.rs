use crate::metrics::ProcessInfo;
use std::collections::HashMap;
use std::time::Instant;

/// Per-PID state for delta-based CPU% calculation.
/// Stores previous cumulative CPU time (Mach absolute units) and wall-clock timestamp.
pub struct ProcessCpuState {
    prev: HashMap<i32, (u64, u64, u64, u64, Instant)>, // (cpu_mach_time, energy_nj, io_read, io_write, timestamp)
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

/// Compute a weighted score for process ranking across multiple dimensions.
/// Normalizes each dimension to 0.0-1.0 against the provided maximums.
/// Score = 0.5 * cpu_norm + 0.3 * mem_norm + 0.2 * power_norm
/// Spike bonus: if any single normalized value > 0.9, add 0.5 to score.
pub fn weighted_score(proc: &ProcessInfo, max_cpu: f32, max_mem: u64, max_power: f32) -> f64 {
    // Division-by-zero guard: if max is 0, norm is 0.0
    let cpu_norm = if max_cpu > 0.0 {
        (proc.cpu_pct / max_cpu).clamp(0.0, 1.0) as f64
    } else {
        0.0
    };
    let mem_norm = if max_mem > 0 {
        (proc.mem_bytes as f64 / max_mem as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let power_norm = if max_power > 0.0 {
        (proc.power_w / max_power).clamp(0.0, 1.0) as f64
    } else {
        0.0
    };

    let mut score = 0.5 * cpu_norm + 0.3 * mem_norm + 0.2 * power_norm;

    // Spike bonus: extremely high in any single dimension
    if cpu_norm > 0.9 || mem_norm > 0.9 || power_norm > 0.9 {
        score += 0.5;
    }

    score
}

/// Enumerate all PIDs via proc_listallpids
fn list_all_pids() -> Vec<i32> {
    // SAFETY: proc_listallpids with null/0 returns the current PID count without writing.
    let count = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }

    // Allocate with headroom
    let capacity = (count as usize) + (count as usize) / 5;
    let mut pids: Vec<i32> = vec![0i32; capacity];
    let buf_size = (capacity * std::mem::size_of::<i32>()) as i32;

    // SAFETY: pids is a valid, properly-sized Vec<i32> buffer; buf_size matches its byte length.
    let actual = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut libc::c_void, buf_size) };
    if actual <= 0 {
        return Vec::new();
    }

    pids.truncate(actual as usize);
    pids
}

/// Get process info via proc_pidinfo (PROC_PIDTASKINFO)
fn get_process_info(pid: i32, cpu_state: &mut ProcessCpuState, now: Instant) -> Option<ProcessInfo> {
    // SAFETY: ProcTaskInfo is repr(C) with no padding requirements beyond zeroed memory.
    let mut task_info: ProcTaskInfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<ProcTaskInfo>() as i32;

    // SAFETY: task_info is a valid, zeroed ProcTaskInfo buffer of correct size.
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
    let (cur_energy_nj, cur_io_read, cur_io_write) = read_process_rusage(pid)
        .unwrap_or((0, 0, 0));

    let (cpu_pct, power_w, io_read_bytes_sec, io_write_bytes_sec) =
        if let Some(&(prev_mach_time, prev_energy, prev_io_r, prev_io_w, prev_wall)) = cpu_state.prev.get(&pid) {
            let delta_task_ns = cpu_state.mach_to_ns(cur_mach_time.saturating_sub(prev_mach_time));
            let delta_wall_ns = now.duration_since(prev_wall).as_nanos() as u64;
            let cpu = if delta_wall_ns > 0 {
                (delta_task_ns as f64 / delta_wall_ns as f64 * 100.0) as f32
            } else {
                0.0
            };
            let power = if delta_wall_ns > 0 {
                cur_energy_nj.saturating_sub(prev_energy) as f32 / delta_wall_ns as f32
            } else {
                0.0
            };
            let delta_secs = delta_wall_ns as f64 / 1_000_000_000.0;
            let io_r = if delta_secs > 0.0 {
                cur_io_read.saturating_sub(prev_io_r) as f64 / delta_secs
            } else {
                0.0
            };
            let io_w = if delta_secs > 0.0 {
                cur_io_write.saturating_sub(prev_io_w) as f64 / delta_secs
            } else {
                0.0
            };
            (cpu, power, io_r, io_w)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

    cpu_state.prev.insert(pid, (cur_mach_time, cur_energy_nj, cur_io_read, cur_io_write, now));

    let mem_bytes = task_info.pti_resident_size;
    let thread_count = task_info.pti_threadnum;

    Some(ProcessInfo {
        pid,
        name,
        cpu_pct,
        mem_bytes,
        energy_nj: cur_energy_nj,
        power_w,
        user,
        thread_count,
        io_read_bytes_sec,
        io_write_bytes_sec,
    })
}

fn get_proc_name(pid: i32) -> String {
    let mut buf = [0u8; 256];
    // SAFETY: buf is a valid 256-byte stack buffer; proc_name writes at most buffersize bytes.
    let ret = unsafe { proc_name(pid, buf.as_mut_ptr() as *mut libc::c_void, 256) };
    if ret <= 0 {
        return String::new();
    }
    let len = ret as usize;
    String::from_utf8_lossy(&buf[..len]).to_string()
}

fn get_proc_user(pid: i32) -> String {
    // Try PROC_PIDT_SHORTBSDINFO first (works in VMs and sandboxed environments)
    // SAFETY: ProcBsdShortInfo is repr(C), all-zero is valid for integer/array fields.
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
    // SAFETY: ProcBsdInfo is repr(C), all-zero is valid for integer/array fields.
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
    // SAFETY: passwd is zeroed before use; getpwuid_r writes into the provided buffer
    // and sets result to null on failure. buf is large enough for typical passwd entries.
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

// --- Energy (proc_pid_rusage) ---

const RUSAGE_INFO_V4: i32 = 4;

/// Padded repr(C) struct matching rusage_info_v4 layout.
/// Key fields at verified byte offsets (from macOS sys/resource.h):
///   ri_diskio_bytesread   at offset 144 (u64)
///   ri_diskio_byteswritten at offset 152 (u64)
///   ri_billed_energy       at offset 264 (u64)
/// Total struct size is 296 bytes.
#[repr(C)]
struct RusageInfoV4 {
    _padding_pre_diskio: [u8; 144],    // 16-byte UUID + 16 u64 fields = 16 + 128 = 144
    ri_diskio_bytesread: u64,           // offset 144
    ri_diskio_byteswritten: u64,        // offset 152
    _padding_mid: [u8; 104],            // 13 fields (QoS/system) * 8 bytes = 104
    ri_billed_energy: u64,              // offset 264
    _rest: [u8; 24],                    // 3 trailing fields * 8 bytes = 24
}

// Compile-time assertions: struct size and field offsets (verified against macOS sys/resource.h).
const _: () = assert!(std::mem::size_of::<RusageInfoV4>() == 296);
const _: () = assert!(std::mem::offset_of!(RusageInfoV4, ri_diskio_bytesread) == 144);
const _: () = assert!(std::mem::offset_of!(RusageInfoV4, ri_diskio_byteswritten) == 152);
const _: () = assert!(std::mem::offset_of!(RusageInfoV4, ri_billed_energy) == 264);

/// Read energy + disk I/O from rusage_info_v4 in a single syscall.
/// Returns (energy_nj, diskio_bytesread, diskio_byteswritten).
fn read_process_rusage(pid: i32) -> Option<(u64, u64, u64)> {
    // SAFETY: RusageInfoV4 is repr(C) with compile-time offset assertions verifying layout.
    // proc_pid_rusage writes exactly sizeof(rusage_info_v4) = 296 bytes into the buffer.
    unsafe {
        let mut ri: RusageInfoV4 = std::mem::zeroed();
        let ret = proc_pid_rusage(pid, RUSAGE_INFO_V4, &mut ri as *mut _ as *mut libc::c_void);
        if ret != 0 {
            return None;
        }
        Some((ri.ri_billed_energy, ri.ri_diskio_bytesread, ri.ri_diskio_byteswritten))
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

// Compile-time assertions: ProcTaskInfo field offsets (from macOS sys/proc_info.h).
const _: () = assert!(std::mem::size_of::<ProcTaskInfo>() == 96);
const _: () = assert!(std::mem::offset_of!(ProcTaskInfo, pti_resident_size) == 8);
const _: () = assert!(std::mem::offset_of!(ProcTaskInfo, pti_total_user) == 16);
const _: () = assert!(std::mem::offset_of!(ProcTaskInfo, pti_total_system) == 24);
const _: () = assert!(std::mem::offset_of!(ProcTaskInfo, pti_threadnum) == 84);

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
    // SAFETY: info is a valid MachTimebaseInfo; mach_timebase_info always succeeds on macOS.
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

    fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut libc::c_void) -> i32;

    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
}
