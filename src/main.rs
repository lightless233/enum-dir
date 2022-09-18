use log::{debug, error};
use std::borrow::Borrow;
use std::process::exit;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, watch};

mod args_parser;
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

    // 任务通道
    let (task_tx, mut task_rx) = watch::channel("");
    let (saver_tx, mut saver_rx): (Sender<String>, Receiver<String>) = mpsc::channel(1024);

    // 启动不同的协成
    let task_builder = tokio::spawn(engines::builder());
    for idx in 1..args.borrow().task_count {
        tokio::spawn(engines::worker(idx));
    }
}
