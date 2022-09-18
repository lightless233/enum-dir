use std::fs;

pub fn init_logger() {
    let log4rs_config = include_str!("../log4rs.yml");
    if fs::read_to_string("./log4rs.yml").is_err() {
        fs::write("./log4rs.yml", &log4rs_config).expect("释放日志配置文件失败！");
    }
    log4rs::init_file("./log4rs.yml", Default::default()).expect("初始化日志系统失败！");
}
