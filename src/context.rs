#[derive(Default, Debug)]
pub struct AppContext {
    pub builder_status: u32,
    pub worker_status: Vec<u32>,
    pub saver_status: u32,
}
