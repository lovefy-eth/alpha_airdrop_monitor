use serde::{Deserialize};
use std::{collections::HashSet, time::Duration};
use teloxide::{prelude::*, types::ChatId, utils::command::BotCommands};
use dotenv::dotenv;

const API_URL: &str = "https://www.binance.info/bapi/defi/v1/friendly/wallet-direct/buw/growth/query-alpha-airdrop";
const TG_CHAT_ID: i64 = -1002842249933; // 替换为你的频道 Chat ID
const INTERVAL_SECS: u64 = 30; // 检查间隔时间（秒）

#[derive(Debug, Deserialize)]
struct AirdropResponse {
    data: Option<Data>,
}

#[derive(Debug, Deserialize)]
struct Data {
    configs: Vec<Config>,
}

#[derive(Debug, Deserialize)]
struct Config {
    configId: String,
    configName: String,
    status: String,
    airdropAmount: f64,
    tokenSymbol: String,
    claimStartTime: i64,
    claimEndTime: i64,
    pointsThreshold: f64,
    contractAddress:String,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "可用命令:")]
enum Command {
    #[command(description = "判断机器人是否在线")]
    Ping,
    #[command(description = "显示帮助")]
    Help,
    #[command(description = "获取当前空投列表")]
    Airdrops,
}

fn load_env() {
    // 先加载 .env
    dotenv::from_filename(".env").ok();
    println!("加载环境变量: {:?}", std::env::var("RUST_ENV"));
    if std::env::var("RUST_ENV").unwrap_or_default() == "development" {
        // 如果是开发环境，加载 .env.development
        dotenv::from_filename(".env.development.local").ok();
    } else {
        // 否则加载 .env.production
        dotenv::from_filename(".env.production.local").ok();
        
    }
}


#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("启动空投监控Bot");
    load_env();
    log::info!("TELOXIDE_TOKEN: {:?}", std::env::var("TELOXIDE_TOKEN"));

    let bot = Bot::from_env();

    // 命令监听
    let bot2 = bot.clone();
    tokio::spawn(async move {
        teloxide::commands_repl(bot2, answer, Command::ty()).await;
    });

    let mut sent_ids = HashSet::new();

    loop {
        match fetch_airdrops().await {
            Ok(configs) => {
                for config in configs {
                    if config.status != "ended" && !sent_ids.contains(&config.configId) {
                        let msg = format!(
                            "📢 新空投上线: {}\nToken: {}\n空投量: {}\n积分门槛：{}\n合约地址：{}\n开始时间: {}\n结束时间: {}\n状态: {}",
                            config.configName,
                            config.tokenSymbol,
                            config.airdropAmount,
                            config.pointsThreshold,
                            config.contractAddress,
                            format_timestamp(config.claimStartTime),
                            format_timestamp(config.claimEndTime),
                            config.status
                        );

                        if let Err(err) = bot.send_message(ChatId(TG_CHAT_ID), msg).await {
                            log::error!("发送TG消息失败: {}", err);
                        } else {
                            sent_ids.insert(config.configId.clone());
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("请求出错: {}", err);
            }
        }

        tokio::time::sleep(Duration::from_secs(INTERVAL_SECS)).await;
    }
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Ping => {
            bot.send_message(msg.chat.id, "pong（在线）").await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, "/ping 是否在线\n/airdrops 获取最近空投列表\n").await?;
        }
        Command::Airdrops => {
            match fetch_airdrops().await {
                Ok(configs) => {
                    if configs.is_empty() {
                        bot.send_message(msg.chat.id, "当前没有可用空投。").await?;
                    } else {
                        let mut text = String::from("当前空投列表：\n");
                        for config in configs.iter().take(10) { // 最多展示10个
                            let line = format!(
                                "• {} ({}): {} {}\n",
                                config.configName,
                                config.tokenSymbol,
                                config.airdropAmount,
                                config.status
                            );
                            text.push_str(&line);
                        }
                        bot.send_message(msg.chat.id, text).await?;
                    }
                }
                Err(err) => {
                    bot.send_message(msg.chat.id, format!("获取空投信息失败: {}", err)).await?;
                }
            }
        }
    }
    Ok(())
}

fn format_timestamp(ms: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "无效时间".to_string())
}

async fn fetch_airdrops() -> Result<Vec<Config>, reqwest::Error> {
    use reqwest::Proxy;

    let mut client_builder = reqwest::Client::builder();

    // 检查 HTTPS_PROXY 或 HTTP_PROXY 环境变量
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("HTTP_PROXY")) {
        if let Ok(proxy) = Proxy::all(&proxy_url) {
            client_builder = client_builder.proxy(proxy);
            log::info!("使用代理: {}", proxy_url);
        } else {
            log::warn!("代理地址无效: {}", proxy_url);
        }
    }

    let client = client_builder.build()?;
    let body = serde_json::json!({
        "page": 1,
        "rows": 20
    });

    let res = client
        .post(API_URL)
        .json(&body)
        .send()
        .await?
        .json::<AirdropResponse>()
        .await?;
    log::info!("获取到 {} 个空投配置", res.data.as_ref().map_or(0, |d| d.configs.len()));

    Ok(res.data.map(|d| d.configs).unwrap_or_default())
}