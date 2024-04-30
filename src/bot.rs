use std::sync::Arc;

use serenity::all::{CreateMessage, Http, MessageBuilder, MessageReference, UserId};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::component::logger::Logger;
use crate::component::message_router::MessageRouterBuilder;
use crate::util::parse_util::{parse_card_from_drop, parse_series_cards};
use crate::wishlist_db::WishlistDB;

pub struct BotState<T> 
    where T: Logger 
{
    wishlist_db: WishlistDB<T>,
    // interaction_manager: InteractionManager,
    logger: Arc<T>
}

const _SOFI_USER_ID:UserId = UserId::new(853629533855809596);
const _SOFU_USER_ID:UserId = UserId::new(950166445034188820);
const _NORI_USER_ID:UserId = UserId::new(742070928111960155);
const _ME_USER_ID :UserId = UserId::new(234822770385485824);



pub async fn start_bot<T>(token:impl AsRef<str>, wishlist_db: WishlistDB<T>, logger: Arc<T>) -> serenity::Client 
    where T: Logger + Send + Sync + 'static
{
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;                      

    // create bot instance
    let state = BotState {
        wishlist_db,
        // interaction_manager: InteractionManager::new(),
        logger: logger.clone()
    };


    let message_router = MessageRouterBuilder::new()
    // Check for wishlisted cards
    .message("check").from_user(_NORI_USER_ID)
        .handler(|state, http, msg| Box::pin(wishlist_check(state, http, msg)))
    // SOFI integration
    // .message().from_user(_SOFI_USER_ID)
    //           .handler()
    // Messages from other users will be parsed for commands
    .command("add").prefix(".wa")
        .description("Add cards to your wishlist")
        .no_bots()
        .handler(|state, http, msg| Box::pin(wishlist_add(state, http, msg)))
    .command("remove").prefix(".wr")
        .description("Remove cards from your wishlist")
        .no_bots()
        .handler(|state, http, msg| Box::pin(wishlist_remove(state, http, msg)))
    .command("ping").prefix(".ping")
        .description("Ping Pong")
        .no_bots()
        .handler(|state, http, msg| Box::pin(ping(state, http, msg)))
    .build(state, logger);


    // Create a new instance of the Client, logging in as a bot.
    let client =
        serenity::Client::builder(&token, intents)
                        .event_handler(message_router).await
                        .expect("Err creating client");

    return client;
}


async fn wishlist_remove<T: Logger>(state: &BotState<T>, http: &Http, msg: &Message) {
    let mut message = MessageBuilder::new();

    match parse_series_cards(&msg.content[3..]) {
        None => { message.push("Something went wrong parsing your command."); },
        Some((series, card_names)) => {
            let user_id = msg.author.id.to_string();

            let res = 
                state.wishlist_db.remove_all_from_wishlist(&user_id, series, card_names).await; 

            match res {
                Ok(amount) => message.push(format!("Removed {amount} card(s) from your wishlist!")),
                Err(err) => {
                    state.logger.log_error(err.to_string());
                    message.push(format!("Something went wrong removing cards from your wishlist."))
                }
            };
        },
    }

    send_response( state 
        , http
        , msg
        , CreateMessage::new().content(message.build())
    ).await;
}

async fn wishlist_check<T: Logger>(state: &BotState<T>, http: &Http, msg: &Message) {
    // We assume SOFU/NORI only sends messages about drops
    //   this might change in the future so be mindful
    //   it may become a TODO here

    let mut message = MessageBuilder::new();
    message.push("A card from your wishlist is dropping!\n");

    let mut wishlist_flag = false;

    for line in msg.content.lines() {
        if let Some((series, card)) = parse_card_from_drop(line) {
            let wishlisted_res = 
                state.wishlist_db.get_wishlisted_users(series, card).await;

            if let Err(why) = wishlisted_res {
                state.logger.log_error(format!("Error retrieving wishlisted users for '{card}•{series}' : {why:?}"));
                continue;
            } 

            let wishlisted = wishlisted_res.unwrap();

            if wishlisted.len() > 0 
            {
                wishlist_flag = true;
                message.push(format!("{}: ", card));

                for user in wishlisted {
                    let user_id = user.parse::<u64>().unwrap();
                    message.mention(&UserId::new(user_id).mention());
                    state.logger.log_info(format!("Pinging user `{user_id}` for card `{card}`"));
                }

                message.push("\n");
            }
        }
    }

    if wishlist_flag {
        send_response(state, http, msg, CreateMessage::new().content(message.build())).await;
    }
}

async fn wishlist_add<T: Logger>(state: &BotState<T>, http: &Http, msg: &Message) {
    let mut message = MessageBuilder::new();

    match parse_series_cards(&msg.content[3..]) {
        None => { message.push("Something went wrong parsing the command."); },
        Some((series, card_names)) => {
            let user_id = msg.author.id.to_string();

            let res = 
                state.wishlist_db.add_all_to_wishlist(&user_id, series, card_names).await;

            match res {
                Ok(added_cards_count) => message.push(format!("Added {added_cards_count} card(s) to your wishlist!")),
                Err(err) =>  {
                    state.logger.log_error(format!(".wa | {}", err.to_string()));
                    message.push(format!("Something went wrong adding cards to your wishlist."))
                }
            };
        },
    }

    send_response(state, http, msg, CreateMessage::new().content(message.build())).await;
}

async fn ping<T: Logger>(_: &BotState<T>, http: &Http, msg: &Message) {
    msg.reply_ping(http, "Pong!").await.unwrap();
}

async fn send_response<T: Logger>(state: &BotState<T>, http: &Http, original_msg: &Message, builder: CreateMessage) {
    // Reply to original message 
    let builder = builder.reference_message(MessageReference::from(original_msg));

    // Try to send
    if let Err(why) = original_msg.channel_id.send_message(http, builder).await {
        state.logger.log_error(format!("Error sending message: {why:?}"));
    };
}




















// #[async_trait]
// impl <T> EventHandler for Bot<T> 
//     where T: Logger + Send + Sync
// {
//     async fn ready(&self, _:Context, _:Ready) {
//         self.logger.log_info("Discord bot ready!");
//     }

//     async fn message(&self, ctx: Context, msg: Message) {
//         // Check for wishlisted cards
//         if msg.author.id == _NORI_USER_ID || msg.author.id == _SOFU_USER_ID //|| msg.author.id == _ME_USER_ID
//         { self.wishlist_check(ctx.http(), &msg).await; }
//         // SOFI integration
//         else if msg.author.id == _SOFI_USER_ID { /* println!("{:?}", msg.embeds); */ }
//         // Messages from other users will be parsed for commands
//         else {
//             // Add to wishlist
//             if msg.content.starts_with(".wa") { 
//                 let t = SystemTime::now();
//                 self.wishlist_add(ctx.http(), &msg).await; 
//                 self.logger.log_info(format!("Served `.wa` for user `{}` in {}ms", msg.author.id, t.elapsed().unwrap().as_millis()));
//             } 
//             // List wishlist
//             // else if msg.content.starts_with(".wl") { 
//             //     let t = SystemTime::now();
//             //     self.wishlist_list(ctx, &msg).await; 
//             //     self.logger.log_info(format!("Served `.wl` for user `{}` in {}ms", msg.author.id, t.elapsed().unwrap().as_millis()));
//             // }
//             // Remove from wishlist
//             else if msg.content.starts_with(".wr") { 
//                 let t = SystemTime::now();
//                 self.wishlist_remove(ctx.http(), &msg).await; 
//                 self.logger.log_info(format!("Served `.wr` for user `{}` in {}ms", msg.author.id, t.elapsed().unwrap().as_millis()));
//             }
//             // Ping - Pong
//             else if msg.content.eq(".ping") { msg.reply_ping(ctx.http(), "Pong!").await.unwrap(); }
//         }
//     }

//     async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        
//         if let Interaction::Component(component) = interaction {
            
//             let user_id = component.user.id;

//             component.defer(&ctx.http).await.unwrap();
//             let message = MessageBuilder::new().push("I'm watching you ").mention(&user_id.mention()).push(" :eyes:").build();
//             component.channel_id.send_message(ctx.http, CreateMessage::new().content(message)).await.unwrap();



//             match component.data.custom_id.as_str() {
//                 "prev" => {
//                     // component.defer(&ctx.http).await.unwrap();
//                     // component.channel_id.send_message(ctx.http, CreateMessage::new().content("Hello")).await.unwrap();
//                     todo!()
//                 }
//                 "next" => {
//                     todo!()
//                 },
//                 custom_id => self.logger.log_warning(format!("Unknown interaction custom_id received: `{custom_id}`"))
//             }
//         }
//     }
// }


// impl <T> Bot<T> 
//     where T: Logger + Send + Sync
// {

//     // async fn wishlist_list(&self, ctx: Context, msg: &Message) {
//         // let response = CreateMessage::new();

//         // let user_id = msg.author.id.to_string();        
//         // let wishlisted_series = self.wishlist_db.get_user_wishlist(&user_id).await;

//         // let flat_series = wishlisted_series[1..min(wishlisted_series.len(), 10)].iter()
//         //     .enumerate()
//         //     .map(|(i, (series_name, _))| format!("`{i}` • {series_name}"))
//         //     .collect::<Vec<String>>()
//         //     .join("\n");
        
        
//         // let embed = CreateEmbed::new()
//         //     .title("Wishlist")
//         //     .description(flat_series)
//         //     .color(0x237feb);

//         // let response = response.add_embed(embed)
//         //     .button(CreateButton::new("prev").label("<-"))
//         //     .button(CreateButton::new("next").label("->"));

//         // self.send_response(ctx, msg, response).await;

//         // self.interaction_manager.add_interaction(msg.author.id);
//     // }

    
// }