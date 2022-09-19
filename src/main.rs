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
    let app_context = Arc::new(Mutex::new(context::AppContext::default()));

    // 任务通道
    let (task_tx, task_rx): (Sender<String>, Receiver<String>) = async_channel::bounded(1024);
    let (saver_tx, saver_rx): (Sender<String>, Receiver<String>) = async_channel::bounded(1024);

    // 启动不同的协程
    // task builder
    {
        app_context.lock().await.builder_status = 1;
    }
    let task_builder_handler = tokio::spawn(engines::builder(
        task_tx.clone(),
        rc_args.clone(),
        Arc::clone(&app_context),
    ));

    // worker
    let mut worker_handlers = vec![];
    for idx in 0..rc_args.task_count {
        {
            app_context.lock().await.worker_status.push(1);
        }
        let _handler = tokio::spawn(engines::worker(
            idx,
            rc_args.clone(),
            task_rx.clone(),
            Arc::clone(&app_context),
        ));
        worker_handlers.push(_handler);
    }

    // saver
    {
        app_context.lock().await.saver_status = 1;
    }
    let saver_handler = tokio::spawn(engines::saver());

    // 等待结束
    task_builder_handler.await;
    for h in worker_handlers {
        h.await;
    }
    saver_handler.await;
}
