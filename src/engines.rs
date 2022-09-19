use std::{cell::RefCell, sync::Arc, vec};

use async_channel::{Receiver, Sender};
use itertools::Itertools;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use crate::{args_parser::AppArgs, context::AppContext};

/**
 * 通过迭代器生成待枚举的文件名，并放到 channel 中
 */
pub async fn builder(
    task_channel: Sender<String>,
    args: Arc<AppArgs>,
    app_context: Arc<AppContext>,
) {
    // 处理 suffix
    let mut suffixes: Vec<String> = vec![];
    if args.empty_suffix {
        suffixes.push("".to_owned());
        suffixes.push("/".to_owned())
    }
    args.suffix
        .split(",")
        .for_each(|it| suffixes.push(format!(".{}", it.trim())));
    debug!("suffixes: {:?}", suffixes);

    // 字符池
    let pool = ('a'..='z')
        .chain('A'..='Z')
        .chain('0'..='9')
        .collect::<Vec<_>>();

    // 按照预定长度生成枚举字符串，并放到 channel 中
    let mut current_length = 0;
    for idx in 1..=args.length {
        let product = (1..=idx).map(|_| pool.iter()).multi_cartesian_product();
        for it in product {
            if idx != current_length {
                info!("length {} build done.", current_length);
                current_length = idx;
            }
            for s in &suffixes {
                let path_name = it.iter().map(|&x| x).join("");
                let task = format!("{}{}", path_name, s);
                // debug!("task: {}", task);
                let result = task_channel.send(task.clone()).await;
                if result.is_err() {
                    warn!(
                        "Error put task to channel, task: {}, error: {:?}",
                        task, result
                    );
                }
            }
        }
    }

    app_context.builder_status = 2;
    info!("builder end!");
}

pub async fn worker(idx: usize, args: Arc<AppArgs>, task_channel: Receiver<String>) {
    debug!("engine worker {} start", idx);
    let target = &args.target;

    loop {
        let task = task_channel.recv().await;
        debug!("worker {} receive {:?}", idx, task);
    }
}

pub async fn saver() {}
