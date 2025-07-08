use teloxide::{prelude::*, types::UpdateKind};

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env.development.local").ok();
    let bot = Bot::from_env();

    // 构建 Dispatcher，支持频道消息和私人聊天消息
    Dispatcher::builder(bot.clone(), dptree::entry()
        .branch(Update::filter_channel_post().endpoint(handle_channel_post))
        .branch(Update::filter_message().endpoint(handle_private_message)))
        .build()
        .dispatch()
        .await;
}

// 处理频道消息
async fn handle_channel_post(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        println!("频道 Chat ID: {}, 内容: {}", msg.chat.id, text);
    }
    Ok(())
}

// 处理私聊消息
async fn handle_private_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if msg.chat.is_private() {
            println!("私聊 Chat ID: {}, 内容: {}", msg.chat.id, text);
            bot.send_message(msg.chat.id, "收到私聊消息").await?;
        }
    }
    Ok(())
}
