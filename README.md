# enum-dir

一款快速枚举目录的小工具，使用 Rust 编写，扫起来真的很快。

用于发现目标站点中可能存在的路径信息，同时支持字典模式和暴力枚举模式。

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
    enum-dir.exe [OPTIONS] --target <target>

OPTIONS:
    -t, --target <target>            待爆破文件的链接，例如 https://example.com/
    -d, --dict <dict>                字典模式，指定此模式后，将禁用枚举模式，如果为空，则使用内置字典
    -l, --length <length>            爆破文件名的最大长度，默认为3 [default: 3]
    -m, --method <method>            枚举时使用的 HTTP 方法，默认为 HEAD [default: HEAD]
    -n, --task-count <task-count>    最大并发数量，默认为25 [default: 25]
    -s, --suffix <suffix>            待枚举的文件后缀，多个后缀使用英文逗号分割，默认为：html,htm,php,zip,tar.gz,tar.bz2
                                     [default: html,htm,php,zip,tar.gz,tar.bz2]
    -e, --empty-suffix               是否枚举空后缀，默认枚举
    -o, --output <output>            输出文件路径 [default: ./enum-dir-result.txt]
    -c, --cookie <cookie>            指定枚举时使用的cookie
    -H, --header <header>            指定枚举时的 http header
        --user-agent <user-agent>    指定扫描时候的UA，默认使用 enum-dir 内置的UA [default: EnumDir/0.0.1]
        --random-user-agent          使用随机的 user-agent，来源于 sqlmap，thanks sqlmap
        --http-retry <http-retry>    当某次请求失败是，重试次数，默认为2 [default: 2]
    -p, --proxy <proxy>              socks5 代理或 http 代理，例如 socks5://127.0.0.1:1080
    -h, --help                       Print help information
    -V, --version                    Print version information
```

## 2.1 字典模式说明
字典为纯文本文件，每行一个，支持以下特殊格式：
```plain
# 井号开头的行为注释行，扫描时将会自动忽略，同时也会忽略空行

# 以下为普通的字典内容
admin/index.php
/admin/index.php.bak

# 如果字典中出现 %EXT% ，则会使用指定的 suffix 依次替换
# 例如：index%EXT%，使用默认 suffix 时，
# 在扫描时会生成如下的列表：
# index.html, index.htm, index.php, index.zip, index.tar.gz, index.tar.bz2
index%EXT%

# 除了 %EXT% 外，还支持以下占位符
# %ALPHA%：使用 a-zA-Z 的字符占位
# %NUMBER%：使用 0-9 的字符占位
# %ALPHANUM%：使用 0-9a-zA-Z 的字符占位
# 例如 foo/%ALPHANUM%%EXT% 将会依次生成：
# foo/0.html
# foo/0.htm
# foo/0.php
# ...
# foo/Z.tar.gz
# foo/Z.tar.bz2
```

## 2.2 使用样例
```shell
# 使用内置字典对目标进行枚举，允许空后缀（内置的字典为 ./dicts/default.txt，如果想使用其他字典，需要手动指定）
$ ./enum-dir -t https://example.com/ -e -d 

# 使用指定字典对目标进行枚举，允许空后缀
$ ./enum-dir -t https://example.com/ -e -d ./dicts/top.txt

# 爆破模式，爆破长度为1-5，允许空后缀，使用50个协程并发
$ ./enum-dir -t https://example.com/ -e -l 5 -n 50

# 爆破模式，指定 HTTP Method 为 GET，并且使用指定的 HTTP 头
$ ./enum-dir -t https://example.com/ -m GET -H "Content-Type: application/json" -H "X-Auth: 11223344"

# 字典模式，内置字典，随机UA，指定输出文件
$ ./enum-dir -t https://example.com/ --random-user-agent -d -o ./output.txt
```

# 3. 支持计划
- ~~使用字典枚举~~
- ~~支持 socks5 代理~~
- ~~支持网络错误重试机制~~
- ~~支持自定义 headers、cookies~~
- ~~字典模式中，支持通过占位符动态生成枚举串~~
- ~~github action 自动构建二进制文件~~
- 性能优化