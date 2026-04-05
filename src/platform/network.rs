use crate::metrics::{NetInterface, NetworkMetrics};
use std::collections::HashMap;

pub struct NetworkState {
    prev: HashMap<String, (u64, u64)>, // name -> (rx_bytes, tx_bytes)
    prev_time: std::time::Instant,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            prev: HashMap::new(),
            prev_time: std::time::Instant::now(),
        }
    }

    pub fn collect(&mut self) -> NetworkMetrics {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.prev_time).as_secs_f64().max(0.001);

        let current = read_interface_bytes();
        let mut interfaces = Vec::new();

        for (name, (rx, tx)) in &current {
            let (rx_rate, tx_rate) = if let Some((prev_rx, prev_tx)) = self.prev.get(name) {
                let drx = rx.saturating_sub(*prev_rx) as f64 / dt;
                let dtx = tx.saturating_sub(*prev_tx) as f64 / dt;
                (drx, dtx)
            } else {
                (0.0, 0.0)
            };

            // Classify interface type
            let iface_type = if name.starts_with("en") {
                "wifi".to_string()
            } else if name.starts_with("lo") {
                continue; // skip loopback
            } else {
                "other".to_string()
            };

            interfaces.push(NetInterface {
                name: name.clone(),
                iface_type,
                rx_bytes_sec: rx_rate,
                tx_bytes_sec: tx_rate,
            });
        }

        self.prev = current;
        self.prev_time = now;

        NetworkMetrics { interfaces }
    }
}

fn read_interface_bytes() -> HashMap<String, (u64, u64)> {
    let mut result = HashMap::new();

    unsafe {
        let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut addrs) != 0 {
            return result;
        }

        let mut cur = addrs;
        while !cur.is_null() {
            let ifa = &*cur;

            // Only look at AF_LINK (data link layer) entries
            if !ifa.ifa_addr.is_null() && (*ifa.ifa_addr).sa_family as i32 == AF_LINK {
                let name = std::ffi::CStr::from_ptr(ifa.ifa_name)
                    .to_string_lossy()
                    .to_string();

                if !ifa.ifa_data.is_null() {
                    let data = ifa.ifa_data as *const IfData;
                    let rx = (*data).ifi_ibytes as u64;
                    let tx = (*data).ifi_obytes as u64;
                    result.insert(name, (rx, tx));
                }
            }

            cur = ifa.ifa_next;
        }

        libc::freeifaddrs(addrs);
    }

    result
}

const AF_LINK: i32 = 18;

#[repr(C)]
struct IfData {
    ifi_type: u8,
    ifi_typelen: u8,
    ifi_physical: u8,
    ifi_addrlen: u8,
    ifi_hdrlen: u8,
    ifi_recvquota: u8,
    ifi_xmitquota: u8,
    ifi_unused1: u8,
    ifi_mtu: u32,
    ifi_metric: u32,
    ifi_baudrate: u32,
    ifi_ipackets: u32,
    ifi_ierrors: u32,
    ifi_opackets: u32,
    ifi_oerrors: u32,
    ifi_collisions: u32,
    ifi_ibytes: u32,
    ifi_obytes: u32,
    ifi_imcasts: u32,
    ifi_omcasts: u32,
    ifi_iqdrops: u32,
    ifi_noproto: u32,
    ifi_recvtiming: u32,
    ifi_xmittiming: u32,
    ifi_lastchange: libc::timeval,
}
