use sysinfo::{CpuExt, System, SystemExt};

/// Agnostic status struct for any limited resource.
pub struct Status {
    total_amount: usize,
    used_amount: usize,
}

impl Status {
    pub fn new(total_amount: usize, used_amount: usize) -> Self {
        Self {
            total_amount,
            used_amount,
        }
    }

    /// Get the total capacity.
    pub fn total(&self) -> usize {
        self.total_amount
    }

    /// Get the used capacity.
    pub fn used(&self) -> usize {
        self.used_amount
    }

    /// Get the capacity utilization as a value between 0 and 1.
    pub fn utilization(&self) -> f64 {
        self.used_amount as f64 / self.total_amount as f64
    }
}

/// Struct to persist system monitoring.
pub struct SystemMetrics {
    sys: System,
}

impl SystemMetrics {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    /// Get the status of the server RAM
    pub fn ram_status(&mut self) -> Status {
        self.sys.refresh_memory();
        Status::new(
            self.sys.total_memory() as usize,
            self.sys.used_memory() as usize,
        )
    }

    /// Get the usage of the server CPU
    pub fn cpu_status(&mut self) -> Vec<Status> {
        self.sys.refresh_cpu(); // refresh_cpu_specifics(refresh_kind)

        let mut cpus = vec![];
        for cpu in self.sys.cpus() {
            cpus.push(Status::new(100, cpu.cpu_usage() as usize));
        }

        cpus
    }
}
