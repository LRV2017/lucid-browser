//! Lightweight process and system metrics for the Lucid Control Center.
use std::time::{Duration, Instant};
use sysinfo::{get_current_pid, ProcessesToUpdate, System};

#[derive(Clone, Copy, Debug, Default)]
pub struct PerformanceSnapshot {
    pub browser_cpu_percent: f32,
    pub browser_memory_bytes: u64,
    pub system_memory_used_bytes: u64,
    pub system_memory_total_bytes: u64,
}

pub struct PerformanceMonitor {
    system: Option<System>,
    pid: Option<sysinfo::Pid>,
    last_refresh: Instant,
    snapshot: PerformanceSnapshot,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        // System::new_all is fallible on unsupported or restricted platforms.
        // Keep the monitor available even when the OS metrics backend cannot start.
        let mut system = System::new_all().ok();
        let pid = get_current_pid().ok();
        if let (Some(system), Some(pid)) = (system.as_mut(), pid) {
            let pids = [pid];
            system.refresh_processes(ProcessesToUpdate::Some(&pids), true);
        }
        if let Some(system) = system.as_mut() {
            system.refresh_memory();
        }
        Self {
            system,
            pid,
            last_refresh: Instant::now() - Duration::from_secs(2),
            snapshot: PerformanceSnapshot::default(),
        }
    }

    pub fn snapshot(&mut self) -> PerformanceSnapshot {
        if self.last_refresh.elapsed() >= Duration::from_millis(900) {
            if let Some(system) = self.system.as_mut() {
                system.refresh_memory();
                if let Some(pid) = self.pid {
                    let pids = [pid];
                    system.refresh_processes(ProcessesToUpdate::Some(&pids), true);
                    if let Some(process) = system.process(pid) {
                        self.snapshot.browser_cpu_percent = process.cpu_usage();
                        self.snapshot.browser_memory_bytes = process.memory();
                    }
                }
                self.snapshot.system_memory_used_bytes = system.used_memory();
                self.snapshot.system_memory_total_bytes = system.total_memory();
            }
            self.last_refresh = Instant::now();
        }
        self.snapshot
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self { Self::new() }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}
