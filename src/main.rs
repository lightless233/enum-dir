use std::process::exit;
use std::sync::Arc;

use log::error;
use tokio::sync::Mutex;

use crate::context::{EnumResult, WorkerStatus};

mod args_parser;
mod context;
mod engines;
mod utils;

#[tokio::main]
async fn main() {
    utils::init_logger();
    let args = match args_parser::parse() {
        Ok(v) => Arc::new(v),
        Err(e) => {
            error!("{}", e);
            exit(-1);
        }
    };

    // 初始化 app context
    let app_context = Arc::new(Mutex::new(context::AppContext::new()));

    // 任务通道
    let (task_tx, task_rx) = async_channel::bounded::<String>(1024);
    let (saver_tx, saver_rx) = async_channel::bounded::<EnumResult>(1024);

    // 启动不同的协程
    // task builder
    {
        app_context.lock().await.builder_status = WorkerStatus::Running;
    }
    let task_builder_handler = tokio::spawn(engines::builder(
        task_tx.clone(),
        Arc::clone(&args),
        Arc::clone(&app_context),
    ));

    // worker
    let mut worker_handlers = vec![];
    for idx in 0..args.task_count {
        {
            app_context
                .lock()
                .await
                .worker_status
                .push(WorkerStatus::Running);
        }
        let _handler = tokio::spawn(engines::worker(
            idx,
            Arc::clone(&args),
            task_rx.clone(),
            saver_tx.clone(),
            Arc::clone(&app_context),
        ));
        worker_handlers.push(_handler);
    }

    // saver
    {
        app_context.lock().await.saver_status = WorkerStatus::Running;
    }
    let saver_handler = tokio::spawn(engines::saver(
        Arc::clone(&app_context),
        Arc::clone(&args),
        saver_rx.clone(),
    ));

    // 等待结束
    let _ = task_builder_handler.await;
    for h in worker_handlers {
        let _ = h.await;
    }
    let _ = saver_handler.await;
}
