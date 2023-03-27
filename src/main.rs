mod app_data;
mod hardware;
mod message;
mod redis;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
};

use app_data::Cli;
use bus::Bus;
use clap::Parser;

use time::{macros::format_description, UtcOffset};
use tracing::{debug, error, info, metadata::LevelFilter, span, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt::time::OffsetTime, prelude::*, EnvFilter};

use crate::app_data::AppConfig;

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = {
        let default_config_path = PathBuf::from("config.json");
        let config_path = cli.config.as_deref().unwrap_or(&default_config_path);
        debug!("从目标路径读取配置文件: {}", config_path.display());

        let config_file = std::fs::read_to_string(config_path)?;
        let config: AppConfig = serde_json::from_str(&config_file)?;
        debug!("解析到的配置文件内容: {:?}", config);

        config
    };

    let exit_required = {
        let should_exit = Arc::new(AtomicBool::new(false));
        let exit_required = should_exit.clone();
        ctrlc::set_handler(move || {
            span!(Level::ERROR, "控制回调").in_scope(|| {
                info!("接受到 Ctrl-C 信号, 准备退出程序运行");
                exit_required.store(true, Ordering::Release);
            });
        })?;
        should_exit
    };

    let (redis_reader_tx, redis_reader_rx) = mpsc::channel();
    let (redis_writer_tx, redis_writer_rx) = mpsc::channel();
    let (port_handler_tx, port_handler_rx) = mpsc::channel();
    let bus = Arc::new(Mutex::new(Bus::new(100)));

    let redis_reader = {
        let exit_required = exit_required.clone();
        let config = config.clone();
        std::thread::spawn(move || {
            span!(Level::ERROR, "订阅线程").in_scope(|| {
                info!("启动 Redis 订阅线程");
                redis::reader::read_redis(exit_required, &config, redis_reader_tx);
            });
        })
    };
    let redis_writer = {
        let exit_required = exit_required.clone();
        let config = config.clone();
        std::thread::spawn(move || {
            span!(Level::ERROR, "发布线程").in_scope(|| {
                info!("启动 Redis 发布线程");
                redis::writer::write_redis(exit_required, &config, redis_writer_rx);
            });
        })
    };
    let port_handler = {
        let exit_required = exit_required.clone();
        let bus = bus.clone();
        std::thread::spawn(move || {
            span!(Level::ERROR, "串口线程").in_scope(|| {
                info!("启动串口线程");
                hardware::port::loop_query(exit_required, port_handler_tx, bus, config);
            });
        })
    };

    while !exit_required.load(Ordering::Acquire) {
        if let Ok(act) = redis_reader_rx.try_recv() {
            if let Ok(mut lock) = bus.lock() {
                debug!("将来自 Redis 的消息 {:?} 广播到所有串口处理线程", act);
                lock.broadcast(act);
            }
        }
        if let Ok(msg) = port_handler_rx.try_recv() {
            debug!("将串口处理线程的消息 {:?} 转发到 Redis", msg);
            redis_writer_tx.send(msg)?;
        }
    }

    redis_reader.join().ok();
    redis_writer.join().ok();
    port_handler.join().ok();

    Ok(())
}

fn get_exe_path() -> PathBuf {
    let mut path = match std::env::current_exe() {
        Ok(exe_path) => exe_path,
        Err(e) => {
            eprintln!("无法获取当前exe文件路径: {}", e);
            std::process::exit(1);
        }
    };
    path.pop();
    path
}

fn set_log_hooks() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(get_exe_path(), "wrench.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 尝试获取UTC时区，否则为东八区，根据时区进行格式化
    let local_time = OffsetTime::new(
        UtcOffset::current_local_offset().unwrap_or(UtcOffset::from_hms(8, 0, 0).unwrap()),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );

    let (stdout_log, file_log) = (
        // 根据debug模式决定是否输出颜色
        if cfg!(debug_assertions) {
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_timer(local_time.clone())
                .with_target(false)
        } else {
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_timer(local_time.clone())
                .with_target(false)
                .with_ansi(false)
        },
        tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_timer(local_time)
            .with_target(false)
            .with_ansi(false)
            .pretty(),
    );

    // 根据debug模式决定默认日志级别，可以通过环境变量设置日志级别
    let filter = if cfg!(debug_assertions) {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy()
    } else {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy()
    };

    tracing_subscriber::registry()
        .with(file_log)
        .with(stdout_log)
        .with(filter)
        .init();

    guard
}

fn main() {
    let guard = set_log_hooks();

    span!(Level::ERROR, "主线程").in_scope(|| {
        if let Err(e) = run() {
            error!("运行错误: {}", e);
            std::process::exit(1);
        }
    });

    drop(guard);
}
