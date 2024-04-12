use serenity::all::{Context, CreateMessage, Message, MessageReference};

pub async fn send_response(ctx: Context, original_msg:Message, builder: CreateMessage) {

    // Reply to original message 
    let builder = builder.reference_message(MessageReference::from(&original_msg));

    // Try to send
    if let Err(why) = original_msg.channel_id.send_message(ctx.http, builder).await {
        println!("Error sending message: {why:?}");
    };
}