use crate::metrics::ThermalMetrics;

/// Stateful temperature collector with cached SMC connection.
pub struct TemperatureState {
    conn: u32,
}

impl TemperatureState {
    /// Open SMC connection. Returns None if SMC is unavailable.
    pub fn new() -> Option<Self> {
        let conn = smc_open()?;
        Some(Self { conn })
    }

    /// Collect temperature metrics using the cached SMC connection.
    pub fn collect(&self) -> ThermalMetrics {
        read_smc_temperatures(self.conn).unwrap_or_default()
    }

    /// Return the SMC connection handle for debug enumeration.
    pub fn conn(&self) -> u32 {
        self.conn
    }
}

impl Drop for TemperatureState {
    fn drop(&mut self) {
        smc_close(self.conn);
    }
}

/// Fallback for when SMC is unavailable — returns default metrics.
pub fn collect_temperature() -> ThermalMetrics {
    ThermalMetrics::default()
}

fn read_smc_temperatures(conn: u32) -> Option<ThermalMetrics> {
    let (cpu_keys_dyn, gpu_keys_dyn, ssd_keys_dyn, battery_keys_dyn) = smc_enumerate_temp_keys(conn);

    let cpu_temps: Vec<f32> = if !cpu_keys_dyn.is_empty() {
        cpu_keys_dyn.iter()
            .filter_map(|k| smc_read_temp(conn, k))
            .filter(|&t| t > 0.0 && t < 130.0)
            .collect()
    } else {
        let cpu_keys = ["TC0P", "TC0C", "TC1C", "TC2C", "TC0F", "Tp09", "Tp0T", "Tp01", "Tp02", "Te01", "Te02"];
        cpu_keys.iter()
            .filter_map(|k| smc_read_temp(conn, k))
            .filter(|&t| t > 0.0 && t < 130.0)
            .collect()
    };

    let gpu_temps: Vec<f32> = if !gpu_keys_dyn.is_empty() {
        gpu_keys_dyn.iter()
            .filter_map(|k| smc_read_temp(conn, k))
            .filter(|&t| t > 0.0 && t < 130.0)
            .collect()
    } else {
        let gpu_keys = ["TG0P", "TG0D", "TG1D", "Tg05", "Tg0f", "Tg0j"];
        gpu_keys.iter()
            .filter_map(|k| smc_read_temp(conn, k))
            .filter(|&t| t > 0.0 && t < 130.0)
            .collect()
    };

    let ssd_temps: Vec<f32> = ssd_keys_dyn.iter()
        .filter_map(|k| smc_read_temp(conn, k))
        .filter(|&t| t > 0.0 && t < 130.0)
        .collect();

    let battery_temps: Vec<f32> = battery_keys_dyn.iter()
        .filter_map(|k| smc_read_temp(conn, k))
        .filter(|&t| t > 0.0 && t < 130.0)
        .collect();

    let cpu_avg = if cpu_temps.is_empty() {
        return None;
    } else {
        cpu_temps.iter().sum::<f32>() / cpu_temps.len() as f32
    };

    let gpu_avg = if gpu_temps.is_empty() {
        cpu_avg
    } else {
        gpu_temps.iter().sum::<f32>() / gpu_temps.len() as f32
    };

    let ssd_avg = if ssd_temps.is_empty() {
        0.0
    } else {
        ssd_temps.iter().sum::<f32>() / ssd_temps.len() as f32
    };

    let battery_avg = if battery_temps.is_empty() {
        0.0
    } else {
        battery_temps.iter().sum::<f32>() / battery_temps.len() as f32
    };

    let fan_speeds = read_fan_speeds(conn);

    Some(ThermalMetrics {
        cpu_avg_c: cpu_avg,
        gpu_avg_c: gpu_avg,
        ssd_avg_c: ssd_avg,
        battery_avg_c: battery_avg,
        fan_speeds,
        available: true,
    })
}

/// Read fan speeds from SMC keys F0Ac, F1Ac, etc.
fn read_fan_speeds(conn: u32) -> Vec<u32> {
    let mut speeds = Vec::new();
    for i in 0..4 {
        let key = format!("F{}Ac", i);
        if let Some(rpm) = smc_read_fan_rpm(conn, &key) {
            if rpm > 0 {
                speeds.push(rpm);
            }
        } else {
            break;
        }
    }
    speeds
}

fn smc_read_fan_rpm(conn: u32, key: &str) -> Option<u32> {
    if key.len() != 4 { return None; }
    let key_bytes: [u8; 4] = [
        key.as_bytes()[0], key.as_bytes()[1], key.as_bytes()[2], key.as_bytes()[3],
    ];
    // SAFETY: SmcKeyData is repr(C, packed) with compile-time size/offset assertions.
    // smc_call communicates with the SMC driver via IOConnectCallStructMethod.
    // conn is a valid IOService connection obtained from smc_open.
    unsafe {
        let mut input = SmcKeyData::zeroed();
        let mut output = SmcKeyData::zeroed();
        input.key = u32::from_be_bytes(key_bytes);
        input.data8 = SMC_CMD_READ_KEYINFO;
        if smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output) != 0 {
            return None;
        }
        let data_size = output.key_info.data_size;
        input = SmcKeyData::zeroed();
        output = SmcKeyData::zeroed();
        input.key = u32::from_be_bytes(key_bytes);
        input.key_info.data_size = data_size;
        input.data8 = SMC_CMD_READ_BYTES;
        if smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output) != 0 {
            return None;
        }
        // fpe2: unsigned 14.2 fixed point (big-endian)
        let raw = ((output.bytes[0] as u16) << 8) | (output.bytes[1] as u16);
        Some((raw as f32 / 4.0) as u32)
    }
}

/// Dynamically enumerate SMC temperature keys via SMC_CMD_READ_INDEX.
/// Returns (cpu_keys, gpu_keys) filtered by prefix and flt /sp78 data type.
/// Returns empty vecs if enumeration fails (caller falls back to static list).
/// Dynamically enumerate SMC temperature keys. Public for debug_info().
/// Dynamically enumerate SMC temperature keys.
/// Returns (cpu_keys, gpu_keys, ssd_keys, battery_keys).
pub fn smc_enumerate_temp_keys(conn: u32) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let total = match smc_read_key_count(conn) {
        Some(n) => n,
        None => return (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
    };

    let mut cpu_keys = Vec::new();
    let mut gpu_keys = Vec::new();
    let mut ssd_keys = Vec::new();
    let mut battery_keys = Vec::new();

    for idx in 0..total {
        // SAFETY: SmcKeyData is zeroed; smc_call reads/writes within the struct bounds.
        unsafe {
            let mut input = SmcKeyData::zeroed();
            let mut output = SmcKeyData::zeroed();
            input.data8 = SMC_CMD_READ_INDEX;
            input.data32 = idx;

            if smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output) != 0 {
                continue;
            }

            let key_bytes = output.key.to_be_bytes();
            let key_str = match std::str::from_utf8(&key_bytes) {
                Ok(s) => s.to_string(),
                Err(_) => continue,
            };

            if !key_str.starts_with('T') {
                continue;
            }

            let mut info_input = SmcKeyData::zeroed();
            let mut info_output = SmcKeyData::zeroed();
            info_input.key = output.key;
            info_input.data8 = SMC_CMD_READ_KEYINFO;

            if smc_call(conn, KERNEL_INDEX_SMC, &mut info_input, &mut info_output) != 0 {
                continue;
            }

            let type_bytes = info_output.key_info.data_type.to_be_bytes();
            let type_str = std::str::from_utf8(&type_bytes).unwrap_or("");

            if type_str != "flt " && type_str != "sp78" {
                continue;
            }

            // Classify by prefix
            if key_str.starts_with("Tp") || key_str.starts_with("Te") || key_str.starts_with("TC") {
                cpu_keys.push(key_str);
            } else if key_str.starts_with("Tg") || key_str.starts_with("TG") {
                gpu_keys.push(key_str);
            } else if key_str.starts_with("Ts") || key_str.starts_with("TH") {
                ssd_keys.push(key_str);
            } else if key_str.starts_with("TB") {
                battery_keys.push(key_str);
            }
        }
    }

    (cpu_keys, gpu_keys, ssd_keys, battery_keys)
}

fn smc_read_key_count(conn: u32) -> Option<u32> {
    // SAFETY: SmcKeyData zeroed; reading #KEY via smc_call to get total key count.
    unsafe {
        let mut input = SmcKeyData::zeroed();
        let mut output = SmcKeyData::zeroed();
        input.key = u32::from_be_bytes([b'#', b'K', b'E', b'Y']);
        input.data8 = SMC_CMD_READ_KEYINFO;

        if smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output) != 0 {
            return None;
        }

        let data_size = output.key_info.data_size;

        input = SmcKeyData::zeroed();
        output = SmcKeyData::zeroed();
        input.key = u32::from_be_bytes([b'#', b'K', b'E', b'Y']);
        input.key_info.data_size = data_size;
        input.data8 = SMC_CMD_READ_BYTES;

        if smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output) != 0 {
            return None;
        }

        Some(u32::from_be_bytes([
            output.bytes[0], output.bytes[1], output.bytes[2], output.bytes[3],
        ]))
    }
}

fn smc_open() -> Option<u32> {
    // SAFETY: IOKit framework calls to find and open the AppleSMC service.
    // All returned handles are checked for validity before use; iterator and
    // service objects are released via IOObjectRelease.
    unsafe {
        let matching = IOServiceMatching(c"AppleSMC".as_ptr());
        if matching.is_null() {
            return None;
        }

        let mut iterator: u32 = 0;
        let kr = IOServiceGetMatchingServices(MASTER_PORT, matching, &mut iterator);
        if kr != 0 {
            return None;
        }

        let mut target_service: u32 = 0;
        loop {
            let service = IOIteratorNext(iterator);
            if service == 0 {
                break;
            }

            let mut name = [0i8; 128];
            IORegistryEntryGetName(service, name.as_mut_ptr());
            let name_str = std::ffi::CStr::from_ptr(name.as_ptr()).to_string_lossy();

            if name_str == "AppleSMCKeysEndpoint" {
                target_service = service;
                break;
            }
            IOObjectRelease(service);
        }
        IOObjectRelease(iterator);

        if target_service == 0 {
            return None;
        }

        let mut conn: u32 = 0;
        let kr = IOServiceOpen(target_service, mach_task_self(), 0, &mut conn);
        IOObjectRelease(target_service);
        if kr != 0 {
            return None;
        }
        Some(conn)
    }
}

fn smc_close(conn: u32) {
    unsafe {
        IOServiceClose(conn);
    }
}

fn smc_read_temp(conn: u32, key: &str) -> Option<f32> {
    if key.len() != 4 {
        return None;
    }

    let key_bytes: [u8; 4] = [
        key.as_bytes()[0],
        key.as_bytes()[1],
        key.as_bytes()[2],
        key.as_bytes()[3],
    ];

    // SAFETY: SmcKeyData is repr(C, packed) with verified layout. Two smc_call rounds:
    // first reads key info (data type + size), second reads the actual value bytes.
    // Decoding uses the returned data type to interpret bytes correctly (sp78/flt/fpe2).
    unsafe {
        // First, get the key info to find the data type
        let mut input = SmcKeyData::zeroed();
        let mut output = SmcKeyData::zeroed();

        input.key = u32::from_be_bytes(key_bytes);
        input.data8 = SMC_CMD_READ_KEYINFO;

        let kr = smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output);
        if kr != 0 {
            return None;
        }

        let data_type = output.key_info.data_type;
        let data_size = output.key_info.data_size;

        // Now read the actual value
        input = SmcKeyData::zeroed();
        output = SmcKeyData::zeroed();

        input.key = u32::from_be_bytes(key_bytes);
        input.key_info.data_size = data_size;
        input.data8 = SMC_CMD_READ_BYTES;

        let kr = smc_call(conn, KERNEL_INDEX_SMC, &mut input, &mut output);
        if kr != 0 {
            return None;
        }

        // Decode based on data type
        let type_bytes = data_type.to_be_bytes();
        let type_str = std::str::from_utf8(&type_bytes).unwrap_or("");

        match type_str {
            "sp78" => {
                // SP78: signed 7.8 fixed point (big-endian)
                let raw = ((output.bytes[0] as u16) << 8) | (output.bytes[1] as u16);
                Some(raw as i16 as f32 / 256.0)
            }
            "flt " => {
                // 32-bit float (big-endian)
                let bytes = [output.bytes[0], output.bytes[1], output.bytes[2], output.bytes[3]];
                Some(f32::from_be_bytes(bytes))
            }
            "fpe2" => {
                // FPE2: unsigned 14.2 fixed point
                let raw = ((output.bytes[0] as u16) << 8) | (output.bytes[1] as u16);
                Some(raw as f32 / 4.0)
            }
            _ => {
                // Try SP78 as default for temperature keys
                let raw = ((output.bytes[0] as u16) << 8) | (output.bytes[1] as u16);
                let val = raw as i16 as f32 / 256.0;
                if val > 0.0 && val < 130.0 {
                    Some(val)
                } else {
                    None
                }
            }
        }
    }
}

unsafe fn smc_call(conn: u32, index: u8, input: &mut SmcKeyData, output: &mut SmcKeyData) -> i32 {
    let in_size = std::mem::size_of::<SmcKeyData>();
    let mut out_size = std::mem::size_of::<SmcKeyData>();

    unsafe {
        IOConnectCallStructMethod(
            conn,
            index as u32,
            input as *const _ as *const libc::c_void,
            in_size,
            output as *mut _ as *mut libc::c_void,
            &mut out_size,
        )
    }
}

// --- SMC data structures ---
const SMC_CMD_READ_BYTES: u8 = 5;
const SMC_CMD_READ_INDEX: u8 = 8;
const SMC_CMD_READ_KEYINFO: u8 = 9;
const KERNEL_INDEX_SMC: u8 = 2;
const MASTER_PORT: u32 = 0;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SmcKeyInfoData {
    data_size: u32,
    data_type: u32,
    data_attributes: u8,
}

// Compile-time assertions: SmcKeyInfoData must be exactly 9 bytes,
// SmcKeyData must be exactly 80 bytes to match the macOS kernel struct layout.
const _: () = assert!(std::mem::size_of::<SmcKeyInfoData>() == 9);
const _: () = assert!(std::mem::size_of::<SmcKeyData>() == 80);
// Field offset assertions (from macOS AppleSMC kernel interface).
const _: () = assert!(std::mem::offset_of!(SmcKeyData, key) == 0);
const _: () = assert!(std::mem::offset_of!(SmcKeyData, data8) == 37);
const _: () = assert!(std::mem::offset_of!(SmcKeyData, data32) == 38);
const _: () = assert!(std::mem::offset_of!(SmcKeyData, bytes) == 48);

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SmcKeyData {
    key: u32,               // offset 0, size 4
    vers: [u8; 6],          // offset 4, size 6
    p_limit_data: [u8; 16], // offset 10, size 16
    key_info: SmcKeyInfoData, // offset 26, size 9
    result: u8,             // offset 35, size 1
    status: u8,             // offset 36, size 1
    data8: u8,              // offset 37, size 1
    data32: u32,            // offset 38, size 4
    _pad: [u8; 6],          // offset 42, size 6 (padding to align bytes at offset 48)
    bytes: [u8; 32],        // offset 48, size 32
}                           // total: 80 bytes

impl SmcKeyData {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// --- IOKit FFI ---
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const i8) -> *mut libc::c_void;
    fn IOServiceGetMatchingServices(main_port: u32, matching: *mut libc::c_void, existing: *mut u32) -> i32;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IORegistryEntryGetName(entry: u32, name: *mut i8) -> i32;
    fn IOServiceOpen(service: u32, owning_task: u32, conn_type: u32, connection: *mut u32) -> i32;
    fn IOServiceClose(connection: u32) -> i32;
    fn IOObjectRelease(object: u32) -> i32;
    fn IOConnectCallStructMethod(
        connection: u32,
        selector: u32,
        input: *const libc::c_void,
        input_size: usize,
        output: *mut libc::c_void,
        output_size: *mut usize,
    ) -> i32;
    fn mach_task_self() -> u32;
}
