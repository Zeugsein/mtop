use crate::metrics::ThermalMetrics;

/// Temperature metrics via SMC (System Management Controller).
/// Reads CPU and GPU temperature sensor keys via IOKit SMC interface.
/// Falls back gracefully to default values if SMC is unavailable.
pub fn collect_temperature() -> ThermalMetrics {
    match read_smc_temperatures() {
        Some(m) => m,
        None => ThermalMetrics::default(),
    }
}

fn read_smc_temperatures() -> Option<ThermalMetrics> {
    let conn = smc_open()?;

    // CPU temperature keys — try multiple common keys
    let cpu_keys = ["TC0P", "TC0C", "TC1C", "TC2C", "TC0F", "Tp09", "Tp0T"];
    let gpu_keys = ["TG0P", "TG0D", "TG1D", "Tg05"];

    let cpu_temps: Vec<f32> = cpu_keys
        .iter()
        .filter_map(|k| smc_read_temp(conn, k))
        .filter(|&t| t > 0.0 && t < 130.0) // filter implausible values
        .collect();

    let gpu_temps: Vec<f32> = gpu_keys
        .iter()
        .filter_map(|k| smc_read_temp(conn, k))
        .filter(|&t| t > 0.0 && t < 130.0)
        .collect();

    smc_close(conn);

    let cpu_avg = if cpu_temps.is_empty() {
        return None;
    } else {
        cpu_temps.iter().sum::<f32>() / cpu_temps.len() as f32
    };

    let gpu_avg = if gpu_temps.is_empty() {
        // GPU temps might not be available on all models; use CPU as fallback
        cpu_avg
    } else {
        gpu_temps.iter().sum::<f32>() / gpu_temps.len() as f32
    };

    Some(ThermalMetrics {
        cpu_avg_c: cpu_avg,
        gpu_avg_c: gpu_avg,
    })
}

fn smc_open() -> Option<u32> {
    unsafe {
        let matching = IOServiceMatching(b"AppleSMC\0".as_ptr() as *const i8);
        if matching.is_null() {
            return None;
        }

        let service = IOServiceGetMatchingService(MASTER_PORT, matching);
        if service == 0 {
            return None;
        }

        let mut conn: u32 = 0;
        let kr = IOServiceOpen(service, libc::mach_task_self(), 0, &mut conn);
        IOObjectRelease(service);

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

    IOConnectCallStructMethod(
        conn,
        index as u32,
        input as *const _ as *const libc::c_void,
        in_size,
        output as *mut _ as *mut libc::c_void,
        &mut out_size,
    )
}

// --- SMC data structures ---
const SMC_CMD_READ_BYTES: u8 = 5;
const SMC_CMD_READ_KEYINFO: u8 = 9;
const KERNEL_INDEX_SMC: u8 = 2;
const MASTER_PORT: u32 = 0;

#[repr(C)]
#[derive(Clone, Copy)]
struct SmcKeyInfoData {
    data_size: u32,
    data_type: u32,
    data_attributes: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SmcKeyData {
    key: u32,
    vers: [u8; 6],
    p_limit_data: [u8; 16],
    key_info: SmcKeyInfoData,
    result: u8,
    status: u8,
    data8: u8,
    data32: u32,
    bytes: [u8; 32],
}

impl SmcKeyData {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// --- IOKit FFI ---
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const i8) -> *mut libc::c_void;
    fn IOServiceGetMatchingService(main_port: u32, matching: *mut libc::c_void) -> u32;
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
}
