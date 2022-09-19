use std::{sync::Arc, time::Duration, vec};

use async_channel::{Receiver, Sender};
use itertools::Itertools;
use log::{debug, info, warn};
use reqwest::ClientBuilder;
use tokio::sync::Mutex;

use crate::{args_parser::AppArgs, context::AppContext};

/**
 * 通过迭代器生成待枚举的文件名，并放到 channel 中
 */
pub async fn builder(
    task_channel: Sender<String>,
    args: Arc<AppArgs>,
    app_context: Arc<Mutex<AppContext>>,
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
                        task,
                        result.unwrap_err().0
                    );
                }
            }
        }
    }

    app_context.lock().await.builder_status = 2;
    info!("builder end!");
}

pub async fn worker(
    idx: usize,
    args: Arc<AppArgs>,
    task_channel: Receiver<String>,
    app_context: Arc<Mutex<AppContext>>,
) {
    debug!("engine worker {} start", idx);
    let target = &args.target;
    let http_client = ClientBuilder::new()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap();

    loop {
        let task = task_channel.try_recv();
        debug!("worker {} receive {:?}", idx, task);

        let url = match task {
            Ok(v) => format!("{}{}", target, v),
            Err(_) => {
                if app_context.lock().await.builder_status == 2 {
                    info!("builder has been stopped, worker {} will stop.", idx);
                    break;
                } else {
                    info!("builder is running, worker {} will running.", idx);
                    continue;
                }
            }
        };
        
        // TODO HTTP Method 通过命令行参数选择，默认 HEAD
        // TODO 添加 socks5 代理配置
        debug!("make request to {}", url);
        match http_client.head(&url).send().await {
            Ok(r) => {
                let code = r.status().as_u16();
                info!("{} {}", code, url);
                // if code != 404 {
                //     // TODO 记录扫描结果
                //     info!("{} {}", code, target);
                // }
            }
            Err(e) => {
                // TODO 发包失败了，重试一下，重试策略放到参数里
                info!("{}", e);
            }
        }

    }

    app_context.lock().await.worker_status[idx] = 2;
}

pub async fn saver() {}
