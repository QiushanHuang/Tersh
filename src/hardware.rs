//! Hardware inventory and optional measurements from one bounded host probe.
//! Missing or invalid counters are unknown, never zero or proof of absence.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HardwareInfo {
    pub model: Option<String>,
    pub architecture: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<u32>,
    pub cpu_threads: Option<u32>,
    pub cpu_usage_percent: Option<u16>,
    pub memory_total_bytes: Option<u64>,
    pub memory_used_bytes: Option<u64>,
    pub memory_basis: Option<String>,
    pub gpus: Vec<GpuInfo>,
    pub gpu_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInfo {
    pub id: String,
    pub name: String,
    pub usage_percent: Option<u16>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temperature_c: Option<i16>,
    pub shared_memory: bool,
}

impl HardwareInfo {
    pub fn parse(output: &str) -> Self {
        let mut hardware = Self::default();
        let (mut cpu_start, mut cpu_end) = (None, None);
        for line in output.lines() {
            let Some((key, raw)) = line.split_once('=') else {
                continue;
            };
            let value = raw.trim();
            match key.trim() {
                "hardware_model" => hardware.model = known_text(value),
                "architecture" => hardware.architecture = known_text(value),
                "cpu_model" => hardware.cpu_model = known_text(value),
                "cpu_cores" => hardware.cpu_cores = positive_count(value),
                "cpu_threads" => hardware.cpu_threads = positive_count(value),
                "cpu_usage" => hardware.cpu_usage_percent = percentage(value),
                "cpu_ticks_start" => cpu_start = cpu_ticks(value),
                "cpu_ticks_end" => cpu_end = cpu_ticks(value),
                "memory_total_bytes" => {
                    hardware.memory_total_bytes = value.parse().ok().filter(|v| *v > 0)
                }
                "memory_used_bytes" => hardware.memory_used_bytes = value.parse().ok(),
                "memory_basis" => hardware.memory_basis = known_text(value),
                "hardware_gpu" if hardware.gpus.len() < 64 => {
                    if let Some(gpu) = GpuInfo::parse(value) {
                        hardware.gpus.push(gpu);
                    }
                }
                "gpu_status" => hardware.gpu_status = known_text(value),
                _ => {}
            }
        }
        if let (Some(start), Some(end)) = (cpu_start, cpu_end) {
            hardware.cpu_usage_percent = cpu_utilization(start, end);
        }
        if let (Some(used), Some(total)) = (hardware.memory_used_bytes, hardware.memory_total_bytes)
            && used > total
        {
            hardware.memory_used_bytes = None;
        }
        hardware
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    pub fn memory_percent(&self) -> Option<u16> {
        ratio(self.memory_used_bytes?, self.memory_total_bytes?)
    }
}

impl GpuInfo {
    fn parse(raw: &str) -> Option<Self> {
        let fields = raw.split('|').map(str::trim).collect::<Vec<_>>();
        if fields.len() != 7 {
            return None;
        }
        let shared_memory = fields[6] == "shared";
        let total = (!shared_memory)
            .then(|| fields[4].parse::<u64>().ok().filter(|v| *v > 0))
            .flatten();
        let used = fields[3]
            .parse::<u64>()
            .ok()
            .filter(|v| total.is_none_or(|total| *v <= total));
        Some(Self {
            id: known_text(fields[0])?,
            name: known_text(fields[1])?,
            usage_percent: percentage(fields[2]),
            memory_used_bytes: used,
            memory_total_bytes: total,
            temperature_c: fields[5]
                .parse::<i16>()
                .ok()
                .filter(|v| (-40..=200).contains(v)),
            shared_memory,
        })
    }
}

fn known_text(value: &str) -> Option<String> {
    (!value.is_empty() && value != "unknown" && value != "N/A").then(|| value.to_owned())
}

fn positive_count(value: &str) -> Option<u32> {
    value.parse::<u32>().ok().filter(|v| *v > 0)
}

fn percentage(value: &str) -> Option<u16> {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && (0.0..=100.0).contains(v))
        .map(|v| v.round() as u16)
}

fn ratio(used: u64, total: u64) -> Option<u16> {
    (total > 0 && used <= total).then(|| ((used as f64 / total as f64) * 100.0).round() as u16)
}

// Linux aggregate counters: user, nice, system, idle, iowait, irq, softirq, steal.
// Guest counters are already included in user/nice and must not be added again.
fn cpu_ticks(value: &str) -> Option<[u64; 8]> {
    let values = value
        .split_whitespace()
        .take(8)
        .map(str::parse)
        .collect::<Result<Vec<u64>, _>>()
        .ok()?;
    values.try_into().ok()
}

fn cpu_utilization(start: [u64; 8], end: [u64; 8]) -> Option<u16> {
    let mut total = 0_u64;
    let mut idle = 0_u64;
    for i in 0..8 {
        let delta = end[i].checked_sub(start[i])?;
        total = total.checked_add(delta)?;
        if i == 3 || i == 4 {
            idle = idle.checked_add(delta)?;
        }
    }
    ratio(total.checked_sub(idle)?, total)
}

pub fn bytes_label(value: Option<u64>) -> String {
    match value {
        Some(bytes) if bytes >= 1024 * 1024 * 1024 => {
            format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
        Some(bytes) => format!("{:.0} MiB", bytes as f64 / (1024.0 * 1024.0)),
        None => "unknown".into(),
    }
}
