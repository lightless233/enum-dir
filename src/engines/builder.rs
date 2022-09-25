use crate::args_parser::AppArgs;
use crate::context::AppContext;
use crate::WorkerStatus;
use async_channel::Sender;
use itertools::Itertools;
use log::{debug, error, info, warn};
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::fs::read_to_string;
use tokio::sync::Mutex;

/**
 * 通过迭代器生成待枚举的文件名，并放到 channel 中
 */
pub async fn builder(
    task_channel: Sender<String>,
    args: Arc<AppArgs>,
    app_context: Arc<Mutex<AppContext>>,
) {
    // 先根据命令行参数，判断使用字典模式还是枚举模式
    // 如果 dict_path 不为 None，则使用字典模式，否则使用枚举模式
    if args.dict_path.is_some() {
        // 字典模式
        dict_builder(
            task_channel,
            &args,
            args.dict_path.as_ref().unwrap().as_str(),
        )
        .await;
    } else {
        // 枚举模式
        enum_builder(task_channel, &args).await;
    }

    app_context.lock().await.builder_status = WorkerStatus::Stop;
    info!("builder end!");
}

fn get_suffix_from_cli(args: &AppArgs) -> Vec<String> {
    let mut suffixes: Vec<String> = vec![];
    if args.empty_suffix {
        suffixes.push("".to_owned());
        suffixes.push("/".to_owned())
    }
    args.suffix
        .split(',')
        .for_each(|it| suffixes.push(format!(".{}", it.trim())));
    debug!("suffixes: {:?}", suffixes);
    suffixes
}

/**
 * 枚举模式生产任务
 */
async fn enum_builder(task_channel: Sender<String>, args: &AppArgs) {
    // 处理 suffix
    let suffixes = get_suffix_from_cli(args);

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
}

/**
 * 字典模式生产任务
 */
async fn dict_builder(task_channel: Sender<String>, args: &AppArgs, dict_path: &str) {
    // 如果这里不提前定义 dict_content 变量，后面的 else 分支会出现悬垂引用，暂时想不到更优雅的方案了
    let dict_content: String;
    let dict_lines = if dict_path.is_empty() || !Path::new(dict_path).exists() {
        info!("未指定字典文件或文件不存在，切换到内置字典...");
        include_str!("../../dicts/default.txt").lines()
    } else {
        // 从文件读
        let read_result = read_to_string(dict_path).await;
        if read_result.is_err() {
            error!("读取字典文件出错，错误：{:?}", read_result.unwrap_err());
            exit(-1);
        }
        dict_content = read_result.unwrap();
        dict_content.lines()
    };

    // 获取 suffixes
    let suffixes = get_suffix_from_cli(args);

    for item in dict_lines {
        // 如果字典中有 %EXT% 则替换为设置的后缀，没有的话就直接发送任务
        if item.contains("%EXT%") {
            for suffix in &suffixes {
                let task = item.replace("%EXT%", suffix);
                let result = task_channel.send(task.clone()).await;
                if result.is_err() {
                    warn!(
                        "Error put task to channel, task: {}, error: {:?}",
                        task,
                        result.unwrap_err().0
                    );
                }
            }
        } else {
            let result = task_channel.send(item.to_owned()).await;
            if result.is_err() {
                warn!(
                    "Error put task to channel, task: {}, error: {:?}",
                    item,
                    result.unwrap_err().0
                );
            }
        };
    }
}
