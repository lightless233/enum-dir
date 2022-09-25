use clap::{App, AppSettings, Arg, ArgAction, ArgMatches, crate_version, value_parser};
use log::debug;

#[derive(Debug, Default)]
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

    // not in cli args.
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
                .default_value("./enum-dir-result.txt")
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
        .get_matches()
}

pub fn parse() -> Result<AppArgs, &'static str> {
    let options = get_arg_matches();
    let mut app_args = AppArgs::default();

    // 解析 target 参数
    // TODO 后续需要更加详细的验证 target 参数，不过现在自己用就比较无所谓
    let mut target = options.get_one::<String>("target").unwrap().to_owned();
    if target.is_empty() {
        return Err("target 不能为空");
    }
    target = if target.starts_with("http://") || target.starts_with("https://") {
        target
    } else {
        // TODO 需要探测真正的协议，如果 http 没有跳转，就都使用 http 协议，如果 http 协议跳转到 https 了，就使用 https 协议
        // 目前为了简单，先添加 http 协议
        format!("http://{}", target)
    };
    app_args.target = if target.ends_with('/') {
        target
    } else {
        format!("{}/", target)
    };

    // 解析是否使用了字典模式
    if options.is_present("dict") {
        let dict_path = options.get_one::<String>("dict").unwrap().to_owned();
        app_args.dict_path = Some(dict_path);
    } else {
        app_args.dict_path = None;
    }

    app_args.length = options.get_one::<usize>("length").unwrap().to_owned();
    app_args.task_count = options.get_one::<usize>("task-count").unwrap().to_owned();
    app_args.suffix = options.get_one::<String>("suffix").unwrap().to_owned();
    app_args.empty_suffix = options.is_present("empty-suffix");
    app_args.output = options.get_one::<String>("output").unwrap().to_owned();

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
