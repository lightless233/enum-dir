use clap::{crate_version, App, AppSettings, Arg, ArgMatches};

#[derive(Debug, Default)]
pub struct AppArgs {
    pub target: String,
    pub task_count: usize,
    pub output: String,
    pub suffix: String,
    pub empty_suffix: bool,
    pub length: usize,
}

fn get_arg_matches() -> ArgMatches {
    App::new("enum-dir")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("brute subdomains in Rust.")
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
            Arg::new("length")
                .short('l')
                .long("length")
                .help("爆破文件名的最大长度，默认为3")
                .default_value("3")
                .takes_value(true),
        )
        .arg(
            Arg::new("task-count")
                .short('n')
                .long("task-count")
                .help("最大并发数量，默认为25")
                .default_value("25")
                .takes_value(true),
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
        ).arg(
        Arg::new("output")
            .short('o')
            .long("output")
            .help("输出文件路径")
            .takes_value(true)
            .default_value("./enum-dir-result.txt")
    ).get_matches()
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

    app_args.length = options
        .get_one::<String>("length")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(3);

    app_args.task_count = options
        .get_one::<String>("task-count")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(25);

    app_args.suffix = options.get_one::<String>("suffix").unwrap().to_owned();
    app_args.empty_suffix = options.is_present("empty-suffix");
    app_args.output = options.get_one::<String>("output").unwrap().to_owned();

    Ok(app_args)
}
