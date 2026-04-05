use crate::metrics::ProcessInfo;

pub fn collect_processes() -> Vec<ProcessInfo> {
    // Use sysctl to enumerate processes and get basic info
    // This is the safe, no-sudo approach
    let output = std::process::Command::new("ps")
        .args(["-axo", "pid,pcpu,rss,user,comm"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut procs = Vec::new();

    for line in text.lines().skip(1) {
        // skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let pid = parts[0].parse::<i32>().unwrap_or(0);
        let cpu_pct = parts[1].parse::<f32>().unwrap_or(0.0);
        let rss_kb = parts[2].parse::<u64>().unwrap_or(0);
        let user = parts[3].to_string();
        let name = parts[4..].join(" ");

        // Extract just the binary name from full path
        let short_name = name.rsplit('/').next().unwrap_or(&name).to_string();

        procs.push(ProcessInfo {
            pid,
            name: short_name,
            cpu_pct,
            mem_bytes: rss_kb * 1024,
            user,
        });
    }

    // Sort by CPU% descending
    procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));

    // Keep top 50
    procs.truncate(50);

    procs
}
