use serde::{Deserialize,Serialize};
use std::{collections::HashSet, time::Duration};
use teloxide::{prelude::*, types::ChatId, utils::command::BotCommands};
use once_cell::sync::Lazy;
use std::env;

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
    deductPoints:f64,
    contractAddress:String,
}

#[derive(Serialize)]
struct WeChatTextMessage {
    msgtype: String,
    text: TextContent,
}

#[derive(Serialize)]
struct TextContent {
    content: String,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "版本(v1.0.0)可用命令:")]
enum Command {
    #[command(description = "判断机器人是否在线")]
    Ping,
    #[command(description = "显示帮助")]
    Help,
    #[command(description = "获取最近空投列表")]
    Airdrops,
    #[command(description = "频道消息测试")]
    MsgTest,
}

pub static BN_API_URL: Lazy<String> = Lazy::new(|| {
    env::var("BN_API_URL").expect("请设置 BN_API_URL 环境变量")
});

pub static TG_CHAT_ID: Lazy<i64> = Lazy::new(|| {
    env::var("TG_CHAT_ID").expect("请设置 TG_CHAT_ID 环境变量")
        .parse::<i64>()
        .expect("TG_CHAT_ID 必须是有效的 i64")
});

pub static WX_WEBHOOK_URL: Lazy<Option<String>> = Lazy::new(|| {
    env::var("WX_WEBHOOK_URL").ok().map(String::from)
});

const TG_DEV_CHAT_ID: i64 = 962210352; // 对一些特殊测试命令只对指定TG用户开放


fn load_env() {
    // 先加载 .env
    dotenv::from_filename(".env").ok();
    log::info!("加载环境变量: {:?}", std::env::var("RUST_ENV"));
    if std::env::var("RUST_ENV").unwrap_or_default() == "development" {
        // 如果是开发环境，加载 .env.development
        dotenv::from_filename(".env.development.local").ok();
    } else {
        // 否则加载 .env.production
        dotenv::from_filename(".env.production.local").ok();
        
    }
}


fn build_airdrop_message(config: &Config) -> String {
    format!(
        "📢 新空投上线: {}\n\
        🪙 Token: {}\n\
        🎁 空投量: {}\n\
        📈 积分门槛: {}\n\
        💸 积分消耗: {}\n\
        📦 合约地址: {}\n\
        🕒 开始时间: {}\n\
        ⏳ 结束时间: {}\n\
        🚦 状态: {}",
        config.configName,
        config.tokenSymbol,
        config.airdropAmount,
        config.pointsThreshold,
        config.deductPoints,
        config.contractAddress,
        format_timestamp(config.claimStartTime),
        format_timestamp(config.claimEndTime),
        config.status
    )
}



pub async fn send_wechat_message(webhook_url: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let msg = WeChatTextMessage {
        msgtype: "text".to_string(),
        text: TextContent {
            content: content.to_string(),
        },
    };

    let res = client
        .post(webhook_url)
        .json(&msg)
        .send()
        .await?;

    if res.status().is_success() {
        log::info!("✅ 微信消息已发送");
    } else {
        log::error!("❌ 微信消息发送失败，状态码：{}", res.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("启动空投监控Bot");
    load_env();
    log::debug!("TELOXIDE_TOKEN: {:?}", std::env::var("TELOXIDE_TOKEN"));

    let bot = Bot::from_env();

    // 命令监听
    let bot2 = bot.clone();
    tokio::spawn(async move {
        teloxide::commands_repl(bot2, answer, Command::ty()).await;
    });

    let mut sent_ids = HashSet::new();

    let tg_chat_id = TG_CHAT_ID.clone();

    loop {
        match fetch_airdrops().await {
            Ok(configs) => {
                for config in configs {
                    if config.status != "ended" && !sent_ids.contains(&config.configId) {
                        let msg = build_airdrop_message(&config);


                        let tg_future = bot.send_message(ChatId(tg_chat_id), msg.clone());

                        let wx_webhook_url = WX_WEBHOOK_URL.clone();
                        let wx_future = async move {
                            if let Some(url) = wx_webhook_url {
                                send_wechat_message(&url, &msg).await.ok();
                            } else {
                                ()
                            }
                        };

                        let (tg_result, _) = tokio::join!(tg_future, wx_future);
                        match tg_result {
                            Ok(_) => {
                                log::info!("✅ 发送TG消息成功: {}", config.configName);
                                sent_ids.insert(config.configId.clone());
                            }
                            Err(err) => {
                                log::error!("❌ 发送TG消息失败: {}", err);
                            }
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
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::MsgTest => {
            if msg.chat.id==ChatId(TG_DEV_CHAT_ID) { //只对指定ID开放频道消息测试
                bot.send_message(ChatId(TG_CHAT_ID.clone()), "这是一个频道消息测试").await?;
                if let Some(webhook_url) = WX_WEBHOOK_URL.clone() {
                    send_wechat_message(&webhook_url, "这是一个微信消息测试").await.unwrap();
                } 
            } else {
                bot.send_message(msg.chat.id, "这是私人聊天测试").await?;
            }
            
        }
        Command::Airdrops => {
            match fetch_airdrops().await {
                Ok(configs) => {
                    if configs.is_empty() {
                        bot.send_message(msg.chat.id, "当前没有可用空投。").await?;
                    } else {
                        let first = configs.first().unwrap();
                        let text: String = build_airdrop_message(first);
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
    use chrono::{TimeZone, Utc, FixedOffset};

    let offset = FixedOffset::east_opt(8 * 3600).unwrap(); // +08:00 中国时间
    Utc.timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.with_timezone(&offset)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string())
        .unwrap_or_else(|| "无效时间".to_string())
}


async fn fetch_airdrops() -> Result<Vec<Config>, reqwest::Error> {
    use reqwest::Proxy;

    let mut client_builder = reqwest::Client::builder();

    // 检查 HTTPS_PROXY 或 HTTP_PROXY 环境变量
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("HTTP_PROXY")) {
        if let Ok(proxy) = Proxy::all(&proxy_url) {
            client_builder = client_builder.proxy(proxy);
            //log::info!("使用代理: {}", proxy_url);
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
        .post(&*BN_API_URL)
        .json(&body)
        .send()
        .await?
        .json::<AirdropResponse>()
        .await?;
    log::info!("获取到 {} 个空投配置", res.data.as_ref().map_or(0, |d| d.configs.len()));

    Ok(res
        .data
        .map(|d| d.configs)
        .unwrap_or_default())
}