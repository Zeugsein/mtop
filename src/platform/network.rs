use crate::metrics::{NetInterface, NetworkMetrics};
use std::collections::HashMap;

pub struct NetworkState {
    prev: HashMap<String, (u64, u64, u64)>, // name -> (rx_bytes, tx_bytes, baudrate)
    prev_time: std::time::Instant,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
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

        for (name, (rx, tx, _baud)) in &current {
            let (rx_rate, tx_rate) = if let Some((prev_rx, prev_tx, _)) = self.prev.get(name) {
                let drx = rx.saturating_sub(*prev_rx) as f64 / dt;
                let dtx = tx.saturating_sub(*prev_tx) as f64 / dt;
                (drx, dtx)
            } else {
                (0.0, 0.0)
            };

            // Classify interface type — en* interfaces are IFT_ETHER (0x06);
            // we cannot distinguish WiFi from wired Ethernet without CoreWLAN,
            // so label as "ethernet" (the more general correct classification).
            let iface_type = if name.starts_with("en") {
                "ethernet".to_string()
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

        // Prefer en* interfaces for baudrate; fall back to global max
        let en_baudrate = current.iter()
            .filter(|(name, _)| name.starts_with("en"))
            .map(|(_, &(_, _, baud))| baud)
            .max()
            .unwrap_or(0);
        let primary_baudrate = if en_baudrate > 0 {
            en_baudrate
        } else {
            current.values()
                .map(|&(_, _, baud)| baud)
                .max()
                .unwrap_or(0)
        };

        self.prev = current;
        self.prev_time = now;

        NetworkMetrics { interfaces, primary_baudrate }
    }
}

fn read_interface_bytes() -> HashMap<String, (u64, u64, u64)> {
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
                    let rx = (*data).ifi_ibytes;
                    let tx = (*data).ifi_obytes;
                    let baudrate = (*data).ifi_baudrate;
                    result.insert(name, (rx, tx, baudrate));
                }
            }

            cur = ifa.ifa_next;
        }

        libc::freeifaddrs(addrs);
    }

    result
}

/// Determine sparkline scale tier (bytes/sec) from interface baudrate (bits/sec).
/// Returns the maximum bytes/sec value for sparkline scaling.
pub fn speed_tier_from_baudrate(baudrate: u64) -> u64 {
    if baudrate >= 1_000_000_000 {
        125_000_000  // 1 Gbps → 125 MB/s
    } else if baudrate >= 100_000_000 {
        12_500_000   // 100 Mbps → 12.5 MB/s
    } else {
        1_250_000    // 10 Mbps fallback → 1.25 MB/s
    }
}

const AF_LINK: i32 = 18;

/// Matches macOS `struct if_data64` from <net/if_var.h>.
/// AF_LINK ifaddr data points to this layout on modern macOS.
/// All counter fields are u64 to avoid truncation on machines with >4GB transferred.
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
    ifi_baudrate: u64,
    ifi_ipackets: u64,
    ifi_ierrors: u64,
    ifi_opackets: u64,
    ifi_oerrors: u64,
    ifi_collisions: u64,
    ifi_ibytes: u64,
    ifi_obytes: u64,
    ifi_imcasts: u64,
    ifi_omcasts: u64,
    ifi_iqdrops: u64,
    ifi_noproto: u64,
    ifi_recvtiming: u32,
    ifi_xmittiming: u32,
    ifi_lastchange: libc::timeval,
}
