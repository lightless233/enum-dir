use std::{sync::Arc, time::Duration, vec};
use std::process::exit;

use async_channel::{Receiver, Sender};
use itertools::Itertools;
use log::{debug, error, info, warn};
use rand::prelude::SliceRandom;
use reqwest::{ClientBuilder, Method};
use tokio::fs::File;
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
    let mut current_length = 1;
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
    info!("length {} build done.", current_length);

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

    // 如果没使用 random user agent，直接在这里把UA写进去
    let mut builder = ClientBuilder::new().timeout(Duration::from_secs(12));
    if !args.random_user_agent {
        builder = builder.user_agent(&args.user_agent);
    }

    // 如果在CLI参数中指定了代理，则把代理设置进去，默认对 http/https 协议都生效
    if let Some(proxy) = &args.proxy {
        let _proxy = reqwest::Proxy::all(proxy);
        if _proxy.is_err() {
            error!("代理设置错误！");
            exit(-1);
        }
        builder = builder.proxy(_proxy.unwrap());
    }

    let http_client = builder.build().unwrap();

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
        // 解析出指定的 HTTP Method
        let method = Method::from_bytes(args.request_method.as_bytes()).unwrap();

        // 如果使用了 random-user-agent 选项，就随机一个 agent 出来，然后塞到头里
        let mut request = http_client.request(method, &url);
        if args.random_user_agent {
            let random_ua = args.user_agent_list.choose(&mut rand::thread_rng());
            request = request.header("User-Agent", random_ua.unwrap());
        }

        // 如果在 CLI 参数中设置了 header 则依次添加
        for header in &args.headers {
            let header_part = header.splitn(2, ':').collect::<Vec<&str>>();

            // 跳过不合法的header
            if header_part.len() < 2 {
                continue;
            }

            let key = header_part[0].trim();
            let value = header_part[1].trim();

            request = request.header(key, value);
        }

        // 如果在 CLI 参数中设置了 cookie 则添加一个 cookie 头
        if let Some(cookie) = &args.cookies {
            request = request.header("Cookie", cookie);
        }

        // 根据重试策略，进行重试
        for c in 0..args.http_retries {
            match request.try_clone().unwrap().send().await {
                Ok(r) => {
                    let code = r.status().as_u16();
                    let result = EnumResult {
                        status_code: code,
                        url: url.clone(),
                    };
                    // TODO 有个问题，如果只发送引用过去会不会性能好一点
                    let _ = result_channel.send(result).await;
                    break;
                }
                Err(e) => {
                    warn!("HTTP Request to {} failed, retry {}, error: {}",url, c+1, e);
                }
            };
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

            info!("Found {} {}", result.status_code, result.url);

            let line = format!("{} {}\n", result.status_code, result.url);
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
