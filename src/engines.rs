use log::debug;

pub async fn builder() {}

pub async fn worker(idx: usize) {
    debug!("engine worker {} start", idx);
}

pub async fn saver() {}
