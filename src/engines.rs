use std::{sync::Arc, time::Duration};
use std::process::exit;

use async_channel::{Receiver, Sender};
use log::{debug, error, info, warn};
use rand::prelude::SliceRandom;
use reqwest::{ClientBuilder, Method};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::{args_parser::AppArgs, context::AppContext, WorkerStatus};
use crate::context::EnumResult;

pub mod builder;
pub use builder::builder;

pub async fn worker(
    idx: usize,
    args: Arc<AppArgs>,
    task_channel: Receiver<String>,
    result_channel: Sender<Arc<EnumResult>>,
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
                    let _ = result_channel.send(Arc::new(result)).await;
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
    result_channel: Receiver<Arc<EnumResult>>,
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
