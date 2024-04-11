use std::time::SystemTime;

use serenity::all::{MessageBuilder, UserId};
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
        if msg.author.id == NORI_USER_ID || msg.author.id == SOFU_USER_ID //|| msg.author.id == _ME_USER_ID
        {
            // We assume SOFU/NORI only sends messages about drops
            //   this might change in the future so be mindful
            //   it may become a TODO here

            let mut response = MessageBuilder::new();
            response.push("A card from your wishlist is dropping!\n");

            let mut wishlist_flag = false;

            for line in msg.content.lines() {
                if let Some((series, card)) = parse_card_from_drop(line) {
                    let wishlisted_res = self.wishlist_db.get_wishlisted_users(series, card).await;
    
                    if let Err(why) = wishlisted_res {
                        println!("Error retrieving wishlisted from '{card}•{series}' : {why:?}");
                        continue;
                    } 
    
                    let wishlisted = wishlisted_res.unwrap();
    
                    if wishlisted.len() > 0 
                    {
                        wishlist_flag = true;
                        response.push(format!("{}: ", card));
    
                        for user in wishlisted {
                            let user_id = user.parse::<u64>().unwrap();
                            response.mention(&UserId::new(user_id).mention());
                        }
    
                        response.push("\n");
                    }
                }
            }

            if !wishlist_flag {
                return;
            }

            send_response(&ctx, msg.channel_id, response.build()).await;
        }
        else if msg.author.id == SOFI_USER_ID
        {
            // println!("{:?}", msg.embeds);
        }
        else
        {
            // Messages from other users will be parsed for commands
            let mut response = MessageBuilder::new();

            if msg.content.starts_with(".wa") {
                // Add to wishlist
                match parse_series_cards(&msg.content[3..]) {
                    Some((series, card_names)) => {
                        let user_id = msg.author.id.to_string();
                        let cards_n = card_names.len();

                        let time = SystemTime::now();
                        let error = self.wishlist_db.add_all_to_wishlist(&user_id, series, card_names).await;
                        println!("Elapsed time: {}ms", time.elapsed().unwrap().as_millis());

                        if error.is_none() {
                            response.push(format!("Added all {} cards to your wishlist!", cards_n));
                        } else {
                            response.push(format!("Something went wrong while adding {} cards to your wishlist.", cards_n));
                            // log errors
                            println!("{}", error.unwrap());
                        };
                    },
                    None => { response.push("Something went wrong parsing the command."); },
                }
            }
            else if msg.content.starts_with(".wl") {
                todo!()
            }
            else if msg.content.starts_with(".wr") {
                // Remove from wishlist
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
                            response.push(format!("Removed all {} cards from your wishlist!", cards_n));
                        } else {
                            let successful = cards_n - errors.len();
                            response.push(format!("Removed {} cards from your wishlist!\n", successful));
                            response.push(format!("Something went wrong while removing {} cards from your wishlist.", errors.len()));

                            // log errors
                            for error in errors {
                                println!("{:?}", error)
                            }
                        };
                    },
                    None => { response.push("Something went wrong parsing the command."); },
                }
            }
            else if msg.content.eq(".ping") 
            {
                response.push("Pong!");
            }

            let message = response.build();
            if !message.is_empty() {
                send_response(&ctx, msg.channel_id, message).await;
            }
        }
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