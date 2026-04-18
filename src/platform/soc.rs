use crate::metrics::SocInfo;
use std::ffi::CStr;

fn sysctl_string(name: &str) -> Option<String> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut size: libc::size_t = 0;
    unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size];
        libc::sysctlbyname(
            cname.as_ptr(),
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        CStr::from_ptr(buf.as_ptr() as *const i8)
            .to_str()
            .ok()
            .map(|s| s.to_string())
    }
}

fn sysctl_u32(name: &str) -> Option<u32> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut val: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as libc::size_t;
    unsafe {
        let ret = libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u32 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if ret == 0 { Some(val) } else { None }
    }
}

fn sysctl_u64(name: &str) -> Option<u64> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut val: u64 = 0;
    let mut size = std::mem::size_of::<u64>() as libc::size_t;
    unsafe {
        let ret = libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u64 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if ret == 0 { Some(val) } else { None }
    }
}

pub fn detect_soc() -> SocInfo {
    let chip = sysctl_string("machdep.cpu.brand_string").unwrap_or_else(|| "Unknown".into());

    // Parse core counts from sysctl
    let total_cores = sysctl_u32("hw.ncpu").unwrap_or(0);
    let p_cores = sysctl_u32("hw.perflevel0.logicalcpu").unwrap_or(0);
    let e_cores = sysctl_u32("hw.perflevel1.logicalcpu").unwrap_or(0);

    // Fallback: if perflevel sysctls don't work, use total as p_cores
    let (e_cores, p_cores) = if e_cores == 0 && p_cores == 0 {
        (0, total_cores)
    } else {
        (e_cores, p_cores)
    };

    // GPU cores - not directly available via sysctl, estimate from chip name
    let gpu_cores = estimate_gpu_cores(&chip);

    let mem_bytes = sysctl_u64("hw.memsize").unwrap_or(0);
    let memory_gb = (mem_bytes / (1024 * 1024 * 1024)) as u32;

    SocInfo {
        chip,
        e_cores,
        p_cores,
        gpu_cores,
        memory_gb,
    }
}

#[allow(clippy::if_same_then_else)]
fn estimate_gpu_cores(chip: &str) -> u32 {
    // Estimate GPU cores from chip model name
    let lower = chip.to_lowercase();
    if lower.contains("ultra") {
        if lower.contains("m4") {
            80
        } else if lower.contains("m3") {
            76
        } else if lower.contains("m2") {
            76
        } else {
            64
        }
    } else if lower.contains("max") {
        if lower.contains("m4") {
            40
        } else if lower.contains("m3") {
            40
        } else if lower.contains("m2") {
            38
        } else {
            32
        }
    } else if lower.contains("pro") {
        if lower.contains("m4") {
            20
        } else if lower.contains("m3") {
            18
        } else if lower.contains("m2") {
            19
        } else {
            16
        }
    } else {
        if lower.contains("m4") {
            10
        } else if lower.contains("m3") {
            10
        } else if lower.contains("m2") {
            10
        } else {
            8
        }
    }
}
