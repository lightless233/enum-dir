use crate::args_parser::AppArgs;
use crate::context::AppContext;
use crate::WorkerStatus;
use async_channel::Sender;
use itertools::Itertools;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use std::vec;
use tokio::fs::read_to_string;
use tokio::sync::Mutex;

/**
 * 合法的 PAT 列表
 */
// static PAT: [&str; 4] = ["%ALPHA%", "%NUMBER%", "%ALPHANUM%", "%EXT%"];

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
                // let path_name = it.iter().map(|&x| x).join("");
                let path_name = it.iter().cloned().join("");
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

    // 为 pattern 构建 pool
    let mut pools = HashMap::new();
    pools.insert(
        "%ALPHA%",
        ('a'..='z')
            .chain('A'..='Z')
            .map(|it| it.to_string())
            .collect::<Vec<String>>(),
    );
    pools.insert(
        "%NUMBER%",
        ('0'..='9')
            .map(|it| it.to_string())
            .collect::<Vec<String>>(),
    );
    pools.insert(
        "%ALPHANUM%",
        ('a'..='z')
            .chain('A'..='Z')
            .chain('0'..='9')
            .map(|it| it.to_string())
            .collect::<Vec<String>>(),
    );
    pools.insert("%EXT%", suffixes);

    for line in dict_lines {
        // 跳过空行和注释行
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 如果字典中的某一项是以 / 开头的，则去掉 / 符号
        let item = if line.starts_with('/') {
            line.trim_start_matches('/')
        } else {
            line
        };

        let line_parts = get_line_part(item);
        let mut tasks: Vec<String> = vec![];
        for pat in line_parts {
            if let Some(pool) = pools.get(pat.as_str()) {
                // 当前部分是占位符
                if tasks.is_empty() {
                    pool.iter().for_each(|it| tasks.push(it.to_owned()));
                } else {
                    let tmp = tasks.clone();
                    let product = tmp.iter().cartesian_product(pool);
                    tasks.clear();
                    product.for_each(|it| tasks.push(format!("{}{}", it.0, it.1)));
                }
            } else {
                // 当前部分不是占位符，直接往 tasks 里塞东西
                if tasks.is_empty() {
                    tasks.push(pat);
                } else {
                    let tmp = tasks.clone();
                    tasks.clear();
                    for item in tmp {
                        tasks.push(format!("{}{}", item, pat));
                    }
                }
            }
        }

        // debug!("tasks: {:?}, line: {}", tasks, line);
        for task in tasks {
            if let Err(e) = task_channel.send(task.clone()).await {
                warn!(
                    "Error put task to channel, line: {}, task: {}, error: {:?}",
                    line, task, e
                );
            }
        }
    }
}

/**
 * 一个小型的状态机，解析字典中的每一行数据，并且将占位符分割出来
 */
fn get_line_part(line: &str) -> Vec<String> {
    // 记录 FSM 当前的状态
    // 0: 在 %XXX% 外面，直接记录每一个字符
    // 1: 在 %XXX% 里面，等到下一次%的时候检查缓冲区里的内容
    let mut status: u8 = 0;

    let mut result: Vec<String> = vec![];
    let mut tmp_buffer: Vec<char> = vec![];

    for c in line.chars() {
        if c == '%' {
            match status {
                0 => {
                    // 开始进入 pat，把 tmp_buffer 清空，开始记录 pat
                    if !tmp_buffer.is_empty() {
                        result.push(tmp_buffer.iter().collect::<String>());
                        tmp_buffer.clear();
                    }
                    tmp_buffer.push(c);
                    status = 1;
                }
                1 => {
                    // pat 结束的标志
                    tmp_buffer.push(c);
                    let t: String = tmp_buffer.iter().collect::<String>();
                    result.push(t);
                    tmp_buffer.clear();
                    status = 0;
                }
                _ => continue,
            }
        } else {
            tmp_buffer.push(c);
        }
    }
    if !tmp_buffer.is_empty() {
        result.push(tmp_buffer.iter().collect::<String>());
    }

    result
}
