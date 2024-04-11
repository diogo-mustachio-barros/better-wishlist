use serenity::all::{ChannelId, Context};

pub async fn send_response(ctx: &Context, channel_id:ChannelId, msg: String) {
    if let Err(why) = channel_id.say(&ctx.http, msg).await {
        println!("Error sending message: {why:?}");
    };
}