use async_channel::{Receiver, Sender};
use log::{debug, error};
use std::cell::RefCell;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;

mod args_parser;
mod context;
mod engines;
mod utils;

#[tokio::main]
async fn main() {
    utils::init_logger();
    let args = match args_parser::parse() {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            exit(-1);
        }
    };
    let rc_args = Arc::new(args);

    // 初始化 app context
    let app_context = Arc::new(context::AppContext::default());

    // 任务通道
    let (task_tx, task_rx): (Sender<String>, Receiver<String>) = async_channel::bounded(1024);
    let (saver_tx, saver_rx): (Sender<String>, Receiver<String>) = async_channel::bounded(1024);

    // 启动不同的协程
    // task builder
    app_context.builder_status = 1;
    let task_builder_handler = tokio::spawn(engines::builder(
        task_tx.clone(),
        rc_args.clone(),
        app_context.clone(),
    ));

    // worker
    debug!("!111");
    t = app_context.clone().lock_owned().await;
    debug!("!2222");
    let mut worker_handlers = vec![];
    for idx in 1..rc_args.task_count {
        t.worker_status[idx] = 1;
        let h = tokio::spawn(engines::worker(idx, rc_args.clone(), task_rx.clone()));
        worker_handlers.push(h);
    }

    // saver
    t = app_context.clone().lock_owned().await;
    t.saver_status = 1;
    let saver_handler = tokio::spawn(engines::saver());

    // 等待结束
    task_builder_handler.await;
}
