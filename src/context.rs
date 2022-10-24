use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct EnumProgressBar {
    pub total: u64,
    pub instance: ProgressBar,
}

impl EnumProgressBar {
    pub fn new(total: u64) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix:>12.cyan.bold} [{bar:57}] {pos}/{len} {wide_msg}",
            )
            .unwrap()
            .progress_chars("=> "),
        );
        pb.set_prefix("Scanning");
        Self {
            total,
            instance: pb,
        }
    }
}

#[derive(Debug)]
pub struct AppContext {
    pub builder_status: WorkerStatus,
    pub worker_status: Vec<WorkerStatus>,
    pub saver_status: WorkerStatus,
    pub pb: Option<EnumProgressBar>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            builder_status: WorkerStatus::Init,
            worker_status: vec![],
            saver_status: WorkerStatus::Init,
            pb: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorkerStatus {
    Init,
    Running,
    Stop,
}

#[derive(Debug, Default)]
pub struct EnumResult {
    pub status_code: u16,
    pub url: String,
    pub content: Option<String>,
}
