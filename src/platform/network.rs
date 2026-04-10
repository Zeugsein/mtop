use crate::metrics::{NetInterface, NetworkMetrics};
use std::collections::HashMap;

pub struct NetworkState {
    prev: HashMap<String, (u64, u64, u64, u64, u64)>, // name -> (rx_bytes, tx_bytes, baudrate, rx_packets, tx_packets)
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

        for (name, (rx, tx, baud, rx_pkts, tx_pkts)) in &current {
            if name.starts_with("lo") {
                continue; // skip loopback
            }

            let (rx_rate, tx_rate, pkt_in_rate, pkt_out_rate) = if let Some((prev_rx, prev_tx, _, prev_rpkt, prev_tpkt)) = self.prev.get(name) {
                let drx = rx.saturating_sub(*prev_rx) as f64 / dt;
                let dtx = tx.saturating_sub(*prev_tx) as f64 / dt;
                let dpkt_in = rx_pkts.saturating_sub(*prev_rpkt) as f64 / dt;
                let dpkt_out = tx_pkts.saturating_sub(*prev_tpkt) as f64 / dt;
                (drx, dtx, dpkt_in, dpkt_out)
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

            let iface_type = classify_interface(name).to_string();

            interfaces.push(NetInterface {
                name: name.clone(),
                iface_type,
                rx_bytes_sec: rx_rate,
                tx_bytes_sec: tx_rate,
                baudrate: *baud,
                packets_in_sec: pkt_in_rate,
                packets_out_sec: pkt_out_rate,
                rx_bytes_total: *rx,
                tx_bytes_total: *tx,
            });
        }

        // Prefer en* interfaces for baudrate; fall back to global max
        let en_baudrate = current.iter()
            .filter(|(name, _)| name.starts_with("en"))
            .map(|(_, &(_, _, baud, _, _))| baud)
            .max()
            .unwrap_or(0);
        let primary_baudrate = if en_baudrate > 0 {
            en_baudrate
        } else {
            current.values()
                .map(|&(_, _, baud, _, _)| baud)
                .max()
                .unwrap_or(0)
        };

        self.prev = current;
        self.prev_time = now;

        NetworkMetrics { interfaces, primary_baudrate }
    }
}

fn read_interface_bytes() -> HashMap<String, (u64, u64, u64, u64, u64)> {
    let mut result = HashMap::new();

    // SAFETY: getifaddrs allocates a linked list of ifaddrs; we iterate it and free with
    // freeifaddrs. AF_LINK entries have ifa_data pointing to if_data64 (IfData layout
    // verified by compile-time offset assertions). All pointers are null-checked before use.
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
                    let rx_pkts = (*data).ifi_ipackets;
                    let tx_pkts = (*data).ifi_opackets;
                    result.insert(name, (rx, tx, baudrate, rx_pkts, tx_pkts));
                }
            }

            cur = ifa.ifa_next;
        }

        libc::freeifaddrs(addrs);
    }

    result
}

fn classify_interface(name: &str) -> &'static str {
    if name.starts_with("en") {
        "Ethernet/Wi-Fi"
    } else if name.starts_with("utun") {
        "VPN"
    } else if name.starts_with("bridge") {
        "Bridge"
    } else if name.starts_with("awdl") {
        "AirDrop"
    } else if name.starts_with("lo") {
        "Loopback"
    } else {
        "Other"
    }
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

// Compile-time assertions: IfData field offsets (from macOS net/if_var.h, struct if_data64).
// 8 × u8 (0-7), u32 mtu (8), u32 metric (12), then u64 fields from offset 16.
const _: () = assert!(std::mem::offset_of!(IfData, ifi_baudrate) == 16);
const _: () = assert!(std::mem::offset_of!(IfData, ifi_ipackets) == 24);
const _: () = assert!(std::mem::offset_of!(IfData, ifi_ibytes) == 64);
const _: () = assert!(std::mem::offset_of!(IfData, ifi_obytes) == 72);
