use once_cell::sync::Lazy;
use std::sync::Mutex;
use log::{LevelFilter, Metadata, Record};

/// Simple logger that stores log lines in memory for later retrieval.
pub struct ExecutionLogger {
    logs: Mutex<Vec<String>>,
}

impl ExecutionLogger {
    const fn new() -> Self {
        Self { logs: Mutex::new(Vec::new()) }
    }

    fn take(&self) -> String {
        let mut logs = self.logs.lock().unwrap();
        let out = logs.join("\n");
        logs.clear();
        out
    }

    fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }
}

impl log::Log for ExecutionLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("{} - {}", record.level(), record.args()));
    }

    fn flush(&self) {}
}

static LOGGER: Lazy<ExecutionLogger> = Lazy::new(|| ExecutionLogger::new());

/// Initialize the global execution logger if not already set.
pub fn init_logger() {
    let _ = log::set_logger(&*LOGGER).map(|()| log::set_max_level(LevelFilter::Info));
}

/// Clear any stored logs.
pub fn clear_logs() {
    LOGGER.clear();
}

/// Retrieve and clear logs collected since the last call.
pub fn take_logs() -> String {
    LOGGER.take()
}

/// Returns the process high water mark (peak RSS) in megabytes on Linux.
#[cfg(target_os = "linux")]
pub fn current_peak_memory_mb() -> u32 {
    use procfs::process::Process;
    if let Ok(proc) = Process::myself() {
        if let Ok(status) = proc.status() {
            if let Some(kb) = status.vmhwm {
                return (kb / 1024) as u32;
            }
        }
    }
    0
}

/// Fallback for non-Linux targets.
#[cfg(not(target_os = "linux"))]
pub fn current_peak_memory_mb() -> u32 {
    0
}
