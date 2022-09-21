use std::{sync::Arc, time::Duration, vec};
use tokio::fs::File;

use async_channel::{Receiver, Sender};
use itertools::Itertools;
use log::{debug, info, warn};
use reqwest::{ClientBuilder, Method};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::context::EnumResult;
use crate::{args_parser::AppArgs, context::AppContext, WorkerStatus};

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
        .split(',')
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

    app_context.lock().await.builder_status = WorkerStatus::Stop;
    info!("builder end!");
}

pub async fn worker(
    idx: usize,
    args: Arc<AppArgs>,
    task_channel: Receiver<String>,
    result_channel: Sender<EnumResult>,
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
        let url = match task {
            Ok(v) => format!("{}{}", target, v),
            Err(_) => {
                if app_context.lock().await.builder_status == WorkerStatus::Stop {
                    break;
                } else {
                    continue;
                }
            }
        };

        // TODO 添加 socks5 代理配置
        let method = Method::from_bytes(args.request_method.as_bytes()).unwrap();
        match http_client.request(method, &url).send().await {
            Ok(r) => {
                let code = r.status().as_u16();
                let result = EnumResult {
                    status_code: code,
                    url,
                };
                // debug!("EnumResult: {:?}", _result);
                let _ = result_channel.send(result).await;
            }
            Err(e) => {
                // TODO 发包失败了，重试一下，重试策略放到参数里
                info!("{}", e);
            }
        }
    }

    app_context.lock().await.worker_status[idx] = WorkerStatus::Stop;
}

pub async fn saver(
    app_context: Arc<Mutex<AppContext>>,
    args: Arc<AppArgs>,
    result_channel: Receiver<EnumResult>,
) {
    let output = &args.output;
    let mut output_file_handler = File::create(output).await.unwrap();
    loop {
        let result = result_channel.try_recv();
        if let Ok(result) = result {
            if result.status_code == 404 {
                continue;
            }

            let line = format!("{} {}\n", result.status_code, result.url);
            info!("Found {}", line);

            let _ = output_file_handler
                .write(line.as_bytes().as_ref())
                .await
                .unwrap();
        } else if !app_context
            .lock()
            .await
            .worker_status
            .contains(&WorkerStatus::Running)
        {
            break;
        } else {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }
    }
    app_context.lock().await.saver_status = WorkerStatus::Stop;
    info!("Save worker stop.");
}
