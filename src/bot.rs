use std::cmp::min;

use serenity::all::{CreateButton, CreateEmbed, CreateMessage, Interaction, MessageBuilder, MessageReference, Ready, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::logger::Logger;
use crate::util::parse_util::{parse_card_from_drop, parse_series_cards};
use crate::wishlist_db::WishlistDB;

struct Bot<T> 
    where T: Logger 
{
    wishlist_db: WishlistDB,
    logger: T
}

const SOFI_USER_ID:UserId = UserId::new(853629533855809596);
const SOFU_USER_ID:UserId = UserId::new(853629533855809596);
const NORI_USER_ID:UserId = UserId::new(742070928111960155);
const _ME_USER_ID :UserId = UserId::new(234822770385485824);


#[async_trait]
impl <T> EventHandler for Bot<T> 
    where T: Logger + Send + Sync
{

    async fn ready(&self, _:Context, _:Ready) {
        self.logger.log_info("Discord bot ready!");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Check for wishlisted cards
        if msg.author.id == NORI_USER_ID || msg.author.id == SOFU_USER_ID //|| msg.author.id == _ME_USER_ID
        { self.wishlist_check(ctx, msg).await; }
        // SOFI integration
        else if msg.author.id == SOFI_USER_ID { /* println!("{:?}", msg.embeds); */ }
        // Messages from other users will be parsed for commands
        else {
            // Add to wishlist
            if msg.content.starts_with(".wa") { self.wishlist_add(ctx, msg).await; } 
            // List wishlist
            else if msg.content.starts_with(".wl") { self.wishlist_list(ctx, msg).await; }
            // Remove from wishlist
            else if msg.content.starts_with(".wr") { self.wishlist_remove(ctx, msg).await; }
            // Ping - Pong
            else if msg.content.eq(".ping") { msg.reply_ping(ctx.http, "Pong!").await.unwrap(); }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // ########## CHATGPT
        if let Interaction::Component(component) = interaction {
            // Check if the interaction is a button click
            match component.data.custom_id.as_str() {
                "prev" | "next" => {
                    // Respond to the button interaction with a message
                    component.defer(&ctx.http).await.unwrap();
                    component.channel_id.send_message(ctx.http, CreateMessage::new().content("Hello")).await.unwrap();
                }
                _ => {} // Handle other button interactions if needed
            }
        }
        // ########## CHATGPT
    }
}

impl <T> Bot<T> 
    where T: Logger
{
    async fn wishlist_remove(&self, ctx: Context, msg: Message) {
        let mut message = MessageBuilder::new();

        match parse_series_cards(&msg.content[3..]) {
            Some((series, card_names)) => {
                let user_id = msg.author.id.to_string();
                let cards_n = card_names.len();
    
                let err = 
                    self.wishlist_db.remove_all_from_wishlist(&user_id, series, card_names).await; 
    
                if err.is_none() {
                    message.push(format!("Removed {cards_n} card(s) from your wishlist!"));
                } else {
                    message.push(format!("Something went wrong removing {cards_n} card(s) from your wishlist."));
    
                    // log errors
                    println!("{:?}", err.unwrap());
                };
            },
            None => { message.push("Something went wrong parsing your command."); },
        }

        send_response( ctx
            , msg
            , CreateMessage::new().content(message.build())
        ).await;
    }

    async fn wishlist_check(&self, ctx: Context, msg: Message) {
        // We assume SOFU/NORI only sends messages about drops
        //   this might change in the future so be mindful
        //   it may become a TODO here

        let mut message = MessageBuilder::new();
        message.push("A card from your wishlist is dropping!\n");

        let mut wishlist_flag = false;

        for line in msg.content.lines() {
            if let Some((series, card)) = parse_card_from_drop(line) {
                let wishlisted_res = 
                    self.wishlist_db.get_wishlisted_users(series, card).await;

                if let Err(why) = wishlisted_res {
                    println!("Error retrieving wishlisted from '{card}•{series}' : {why:?}");
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
                    }

                    message.push("\n");
                }
            }
        }

        if wishlist_flag {
            send_response( ctx
                         , msg
                         , CreateMessage::new().content(message.build())
            ).await;
        }
    }

    async fn wishlist_add(&self, ctx: Context, msg: Message) {
        let mut message = MessageBuilder::new();

        match parse_series_cards(&msg.content[3..]) {
            Some((series, card_names)) => {
                let user_id = msg.author.id.to_string();
                let cards_n = card_names.len();

                let error = 
                    self.wishlist_db.add_all_to_wishlist(&user_id, series, card_names).await;

                if error.is_none() {
                    message.push(format!("Added {cards_n} card(s) to your wishlist!"));
                } else {
                    message.push(format!("Something went wrong adding {cards_n} card(s) to your wishlist."));
                    // log errors
                    println!("{}", error.unwrap());
                };
            },
            None => { message.push("Something went wrong parsing the command."); },
        }

        send_response( ctx
            , msg
            , CreateMessage::new().content(message.build())
        ).await;
    }

    async fn wishlist_list(&self, ctx: Context, msg: Message) {
        let response = CreateMessage::new();

        let user_id = msg.author.id.to_string();        
        let wishlisted_series = self.wishlist_db.get_user_wishlist(&user_id).await;

        let flat_series = wishlisted_series[1..min(wishlisted_series.len(), 10)].iter()
            .enumerate()
            .map(|(i, (series_name, _))| format!("`{i}` • {series_name}"))
            .collect::<Vec<String>>()
            .join("\n");
        
        
        let embed = CreateEmbed::new()
            .title("Wishlist")
            .description(flat_series)
            .color(0x237feb);

        let response = response.add_embed(embed)
            .button(CreateButton::new("prev").label("<-"))
            .button(CreateButton::new("next").label("->"));

        send_response(ctx, msg, response).await;
    }
}

pub async fn init_discord_bot<T>(token:impl AsRef<str>, wishlist_db: WishlistDB, logger: T) -> serenity::Client 
    where T: Logger + Send + Sync + 'static
{
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // create bot instance
    let bot = Bot {
        wishlist_db,
        logger
    };

    // Create a new instance of the Client, logging in as a bot.
    let client =
        serenity::Client::builder(&token, intents)
                         .event_handler(bot).await
                         .expect("Err creating client");

    return client;
}

async fn send_response(ctx: Context, original_msg:Message, builder: CreateMessage) {
    // Reply to original message 
    let builder = builder.reference_message(MessageReference::from(&original_msg));

    // Try to send
    if let Err(why) = original_msg.channel_id.send_message(ctx.http, builder).await {
        println!("Error sending message: {why:?}");
    };
}