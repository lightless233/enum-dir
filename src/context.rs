#[derive(Debug)]
pub struct AppContext {
    pub builder_status: WorkerStatus,
    pub worker_status: Vec<WorkerStatus>,
    pub saver_status: WorkerStatus,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            builder_status: WorkerStatus::INIT,
            worker_status: vec![],
            saver_status: WorkerStatus::INIT,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum WorkerStatus {
    INIT,
    RUNNING,
    STOP,
}

#[derive(Debug, Default)]
pub struct EnumResult {
    pub status_code: u16,
    pub url: String,
}
