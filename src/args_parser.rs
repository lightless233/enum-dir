use clap::{crate_version, value_parser, App, AppSettings, Arg, ArgAction, ArgMatches};
use derivative::Derivative;
use log::debug;
use tldextract::TldOption;
use url::Host;

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct AppArgs {
    pub target: String,
    pub task_count: usize,
    pub request_method: String,
    pub output: String,
    pub suffix: String,
    pub empty_suffix: bool,
    pub length: usize,
    pub user_agent: String,
    pub random_user_agent: bool,
    pub cookies: Option<String>,
    pub headers: Vec<String>,
    pub http_retries: usize,
    pub proxy: Option<String>,
    pub dict_path: Option<String>,
    pub black_words: Option<String>,
    pub fixed_length: bool,
    pub debug_mode: bool,

    // not in cli args.
    #[derivative(Debug = "ignore")]
    pub user_agent_list: Vec<String>,
}

fn get_arg_matches() -> ArgMatches {
    App::new("enum-dir")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("Enum dir/path/file on target URL.")
        .version(crate_version!())
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .help("待爆破文件的链接，例如 https://example.com/")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("dict")
                .short('d')
                .long("dict")
                .help("字典模式，指定此模式后，将禁用枚举模式，如果为空，则使用内置字典")
                .takes_value(true)
                .default_missing_value(""),
        )
        .arg(
            Arg::new("length")
                .short('l')
                .long("length")
                .help("爆破文件名的最大长度，默认为3")
                .default_value("3")
                .takes_value(true)
                .value_parser(value_parser!(usize)),
        )
        .arg(
            Arg::new("fixed-length")
                .long("fixed-length")
                .help("固定枚举长度，而非枚举 1..=length ")
                .takes_value(false)
        )
        .arg(
            Arg::new("method")
                .short('m')
                .long("method")
                .help("枚举时使用的 HTTP 方法，默认为 HEAD")
                .default_value("HEAD")
                .takes_value(true),
        )
        .arg(
            Arg::new("task-count")
                .short('n')
                .long("task-count")
                .help("最大并发数量，默认为25")
                .default_value("25")
                .takes_value(true)
                .value_parser(value_parser!(usize)),
        )
        .arg(
            Arg::new("suffix")
                .short('s')
                .long("suffix")
                .help("待枚举的文件后缀，多个后缀使用英文逗号分割，默认为：html,htm,php,zip,tar.gz,tar.bz2")
                .takes_value(true)
                .default_value("html,htm,php,zip,tar.gz,tar.bz2")
        )
        .arg(
            Arg::new("empty-suffix")
                .short('e')
                .long("empty-suffix")
                .help("是否枚举空后缀，默认枚举")
                .takes_value(false)
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("输出文件路径")
                .takes_value(true)
        )
        .arg(
            Arg::new("cookie")
                .short('c')
                .long("cookie")
                .help("指定枚举时使用的cookie")
                .takes_value(true)
        )
        .arg(
            Arg::new("header")
                .action(ArgAction::Append)
                .short('H')
                .long("header")
                .help("指定枚举时的 http header")
                .takes_value(true)
                .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("user-agent")
                .long("user-agent")
                .help("指定扫描时候的UA，默认使用 enum-dir 内置的UA")
                .takes_value(true)
                .default_value("EnumDir/0.0.1")
        )
        .arg(
            Arg::new("random-user-agent")
                .long("random-user-agent")
                .help("使用随机的 user-agent，来源于 sqlmap，thanks sqlmap")
                .takes_value(false)
        )
        .arg(
            Arg::new("http-retry")
                .long("http-retry")
                .help("当某次请求失败是，重试次数，默认为2")
                .takes_value(true)
                .default_value("2")
                .value_parser(value_parser!(usize))
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .short('p')
                .takes_value(true)
                .help("socks5 代理或 http 代理，例如 socks5://127.0.0.1:1080")
        )
        .arg(
            Arg::new("black-words")
                .long("black-words")
                .takes_value(true)
                .help("黑名单关键字，默认为空，设置后当页面内容出现指定的关键字时，认为页面不存在，不记录到结果中。开启该功能后，自动切换为 GET 方法。")
        )
        .arg(
            Arg::new("debug")
            .long("debug")
            .takes_value(false)
            .help("调试模式")
        )
        .get_matches()
}

async fn extract_target(raw_target: &String) -> Result<String, &'static str> {
    if raw_target.is_empty() {
        return Err("target 不能为空");
    }

    // 填充协议，校验 URL 是否合法
    let mut auto_detect = false;
    let tmp_target = if raw_target.starts_with("http://") || raw_target.starts_with("https://") {
        raw_target.clone()
    } else {
        auto_detect = true;
        format!("http://{}", raw_target)
    };
    let uri = reqwest::Url::parse(&tmp_target).unwrap();
    match uri.host().unwrap() {
        Host::Ipv4(_) => {}
        _ => {
            let tld_extractor = TldOption::default().cache_path(".tld_cache").build();
            let tld_result = tld_extractor.extract(&tmp_target).unwrap();
            if tld_result.suffix.is_none() {
                return Err("target有误!");
            }
        }
    };

    // 自动探测协议
    let target = if auto_detect {
        // 需要探测真正的协议，如果 http 没有跳转，就都使用 http 协议，如果 http 协议跳转到 https 了，就使用 https 协议
        debug!("目标中未提供协议，自动探测...");
        let client = reqwest::Client::builder().build().unwrap();
        let http_url = format!("http://{}", raw_target);
        let response = client.head(http_url).send().await.unwrap();
        let schema = response.url().scheme();
        debug!("使用 {} 协议", schema);
        format!("{}://{}", schema, raw_target)
    } else {
        raw_target.clone()
    };

    if target.ends_with('/') {
        Ok(target)
    } else {
        Ok(format!("{}/", target))
    }
}

pub async fn parse() -> Result<AppArgs, &'static str> {
    let options = get_arg_matches();
    let mut app_args = AppArgs::default();

    // 解析 target 参数
    let target = options.get_one::<String>("target").unwrap().to_owned();
    app_args.target = extract_target(&target).await?;

    // 解析是否使用了字典模式
    if options.is_present("dict") {
        let dict_path = options.get_one::<String>("dict").unwrap().to_owned();
        app_args.dict_path = Some(dict_path);
    } else {
        app_args.dict_path = None;
    }

    app_args.length = options.get_one::<usize>("length").unwrap().to_owned();
    app_args.fixed_length = options.is_present("fixed-length");
    app_args.task_count = options.get_one::<usize>("task-count").unwrap().to_owned();
    app_args.suffix = options.get_one::<String>("suffix").unwrap().to_owned();
    app_args.empty_suffix = options.is_present("empty-suffix");
    app_args.output = if let Some(o) = options.get_one::<String>("output") {
        o.to_owned()
    } else {
        // 用户没有指定，使用 target 自动生成
        let filename = app_args
            .target
            .clone()
            .replace("https://", "")
            .replace("http://", "")
            .replace('/', "_")
            .trim_matches('_')
            .to_owned();
        format!("{}.txt", filename)
    };

    // 检查 method 是否合法
    let method = options
        .get_one::<String>("method")
        .unwrap()
        .to_owned()
        .to_uppercase();
    let available_methods = [
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "PATCH", "TRACE",
    ];
    if available_methods.contains(&method.as_str()) {
        app_args.request_method = method;
    } else {
        return Err("method 错误！");
    }

    // 设置 black words
    let black_words = options.get_one::<String>("black-words");
    if black_words.is_some() {
        app_args.black_words = black_words.cloned();
        app_args.request_method = "GET".to_owned();
    }

    // 获取 UA
    app_args.user_agent = options.get_one::<String>("user-agent").unwrap().to_owned();

    // 获取随机 UA 的设置，默认为 false
    app_args.random_user_agent = options.is_present("random-user-agent");
    if app_args.random_user_agent {
        app_args.user_agent_list = read_user_agent();
    }

    // 获取 cookie
    let cookie = options.get_one::<String>("cookie").cloned();
    app_args.cookies = cookie;

    // 获取 headers
    // let headers: Option<ValuesRef<String>> = options.try_get_many("header").unwrap();
    if let Some(header) = options.try_get_many("header").unwrap() {
        let headers = header.collect::<Vec<&String>>();
        for h in headers {
            app_args.headers.push(h.to_owned());
        }
    }

    // http 重试次数
    let http_retries = options.get_one::<usize>("http-retry").unwrap();
    app_args.http_retries = http_retries.to_owned();

    // 代理设置
    let proxy = options.get_one::<String>("proxy");
    app_args.proxy = proxy.cloned();

    app_args.debug_mode = options.is_present("debug");

    debug!("app_args: {:?}", app_args);
    Ok(app_args)
}

fn read_user_agent() -> Vec<String> {
    let mut result = vec![];
    let content = include_str!("../user-agents.txt");
    for mut line in content.lines() {
        line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        result.push(line.to_owned());
    }

    result
}
