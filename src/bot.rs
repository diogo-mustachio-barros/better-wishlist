use std::time::SystemTime;

use serenity::all::{CreateEmbed, CreateMessage, MessageBuilder, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::bot_util::send_response;
use crate::util::{parse_card_from_drop, parse_series_cards};
use crate::wishlist_db::WishlistDB;

struct Bot {
    wishlist_db: WishlistDB
}

const SOFI_USER_ID:UserId = UserId::new(853629533855809596);
const SOFU_USER_ID:UserId = UserId::new(853629533855809596);
const NORI_USER_ID:UserId = UserId::new(742070928111960155);
const _ME_USER_ID :UserId = UserId::new(234822770385485824);


#[async_trait]
impl EventHandler for Bot {
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
}

impl Bot {
    async fn wishlist_remove(&self, ctx: Context, msg: Message) {
        let mut message = MessageBuilder::new();

        match parse_series_cards(&msg.content[3..]) {
            Some((series, card_names)) => {
                let user_id = msg.author.id.to_string();
                let cards_n = card_names.len();
    
                let mut errors: Vec<(&str, mongodb::error::Error)> = vec![];
    
                for card_name in card_names {
                    // println!("'{}'", card_name);
                    if let Some(err) = self.wishlist_db.remove_from_wishlist(
                        &user_id
                        , series.trim()
                        , &card_name
                        ).await 
                    {
                        errors.push((&card_name, err));
                    }
                }
    
                if errors.is_empty() {
                    message.push(format!("Removed all {} cards from your wishlist!", cards_n));
                } else {
                    message.push(format!("Something went wrong while removing {} cards from your wishlist.", errors.len()));
    
                    // log errors
                    for error in errors {
                        println!("{:?}", error)
                    }
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

        if !wishlist_flag {
            return;
        }

        send_response( ctx
                     , msg
                     , CreateMessage::new().content(message.build())
        ).await;
    }

    async fn wishlist_add(&self, ctx: Context, msg: Message) {
        let mut message = MessageBuilder::new();

        match parse_series_cards(&msg.content[3..]) {
            Some((series, card_names)) => {
                let user_id = msg.author.id.to_string();
                let cards_n = card_names.len();

                let time = SystemTime::now();
                let error = self.wishlist_db.add_all_to_wishlist(&user_id, series, card_names).await;
                println!("Elapsed time: {}ms", time.elapsed().unwrap().as_millis());

                if error.is_none() {
                    message.push(format!("Added all {} cards to your wishlist!", cards_n));
                } else {
                    message.push(format!("Something went wrong while adding {} cards to your wishlist.", cards_n));
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
                
                let series = [
                    ( 1, "Series 1"),
                    ( 2, "Series 2"),
                    ( 3, "Series 3"),
                    ( 4, "Series 4"),
                    ( 5, "Series 5"),
                    ( 6, "Series 6"),
                    ( 7, "Series 7"),
                    ( 8, "Series 8"),
                    ( 9, "Series 9"),
                    (10, "Series 10"),
                ];

                let flat_series = series.map(|(order, name)| format!("{order} • {name}")).join("\n");
                
                
                let embed = CreateEmbed::new()
                    .title("Wishlist")
                    .description(flat_series);

                let response = response.add_embed(embed);

                // msg.reply_ping(ctx.http, |m| { });
                // msg.channel_id.send_message(ctx.http, |m : MessageBuilder| {
                //     m.embed(|e| {
                //         e.title("This is a title")
                //             .description("This is a description")
                //             .field("Field 1", "Value 1", true)
                //             .field("Field 2", "Value 2", true)
                //             .footer(|f| f.text("This is a footer"))
                //     })
                // });

                send_response(ctx, msg, response).await;
    }
}

pub async fn init_discord_bot(token:impl AsRef<str>, wishlist_db: WishlistDB) -> serenity::Client {
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // create bot instance
    let bot = Bot {
        wishlist_db
    };

    // Create a new instance of the Client, logging in as a bot.
    let client =
        serenity::Client::builder(&token, intents)
                         .event_handler(bot).await
                         .expect("Err creating client");

    return client;
}