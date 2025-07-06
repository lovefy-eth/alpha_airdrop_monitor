use teloxide::{prelude::*, types::UpdateKind};

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env.development.local").ok();
    let bot = Bot::from_env();

    // 手动实现 dispatcher 以处理频道消息
    Dispatcher::builder(bot, Update::filter_channel_post().endpoint(handle_channel_post))
        .build()
        .dispatch()
        .await;
}

async fn handle_channel_post(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        println!("频道 Chat ID: {}, 内容: {}", msg.chat.id, text);
    }
    Ok(())
}
