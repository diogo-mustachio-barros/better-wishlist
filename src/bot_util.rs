use serenity::all::{Context, Message};

pub async fn send_response(ctx: &Context, original_msg:Message, content: String) {
    if let Err(why) = original_msg.reply_ping(&ctx.http, content).await {
        println!("Error sending message: {why:?}");
    };
}