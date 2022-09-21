# enum-dir

一款快速枚举目录的小工具，使用 Rust 编写，扫起来真的很快。

用于发现目标站点中可能存在的路径信息。

> 本工具仅用于学习 Rust 以及 Rust 协程 Tokio 使用，严禁用于非法用途。
> 
> 如果使用本工具从事违法犯罪活动，造成的任何后果，本人不承担任何责任。

# 1. 开始使用
## 1.1 方法1：手动编译
```shell
git clone https://github.com/lightless233/enum-dir.git
cd enum-dir
cargo build --release 
./target/release/enum-dir
```

## 1.2 方法2：直接下载构建好的二进制文件
```shell
TODO
```

# 2. 参数说明
```shell
USAGE:
    enum-dir [OPTIONS] --target <target>

OPTIONS:
    -t, --target <target>            待爆破文件的链接，例如 https://example.com/
    -l, --length <length>            爆破文件名的最大长度，默认为3 [default: 3]
    -m, --method <method>            枚举时使用的 HTTP 方法，默认为 HEAD [default: HEAD]
    -n, --task-count <task-count>    最大并发数量，默认为25 [default: 25]
    -s, --suffix <suffix>            待枚举的文件后缀，多个后缀使用英文逗号分割，默认为：html,htm,php,zip,tar.gz,tar.bz2
                                     [default: html,htm,php,zip,tar.gz,tar.bz2]
    -e, --empty-suffix               是否枚举空后缀，默认枚举
    -o, --output <output>            输出文件路径 [default: ./enum-dir-result.txt]
        --user-agent <user-agent>    指定扫描时候的UA，默认使用 enum-dir 内置的UA [default:
                                     EnumDir/0.0.1]
        --random-user-agent          使用随机的 user-agent，来源于 sqlmap，thanks sqlmap
    -h, --help                       Print help information
    -V, --version                    Print version information
```

# 3. 支持计划
- 使用字典枚举
- 支持 socks5 代理
- 支持网络错误重试机制
- 支持自定义 headers、cookies