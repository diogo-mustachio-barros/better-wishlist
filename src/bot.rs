use std::sync::Arc;

use poise::serenity_prelude as serenity;

use ::serenity::all::{ReactionCollector, ReactionType};
use ::serenity::all::{CreateMessage, Message, MessageBuilder, MessageReference};
use serenity::all::UserId;
use serenity::prelude::*;

use crate::components::logger::StdoutLogger;
use crate::commands::*;
use crate::integrations::*;
use crate::util::either::Either;
use crate::util::parse_util::{parse_series_card_from_analysis, parse_series_from_analysis};
use crate::wishlist_db::WishlistDB;
use crate::components::logger::Logger;

pub const _SOFI_USER_ID: UserId = UserId::new(853629533855809596);
pub const _SOFU_USER_ID: UserId = UserId::new(950166445034188820);
pub const _NORI_USER_ID: UserId = UserId::new(742070928111960155);
pub const _ME_USER_ID  : UserId = UserId::new(234822770385485824);
pub const _BOT_USER_ID : UserId = UserId::new(1219361361348530298);

pub struct Data {
    pub wishlist_db: WishlistDB<StdoutLogger>,
    pub logger: Arc<StdoutLogger>
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn start_bot(token:impl AsRef<str>, wishlist_db: WishlistDB<StdoutLogger>, logger: Arc<StdoutLogger>) -> serenity::Client
{
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;                      

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                // Commands
                command_wa(), 
                command_wr(), 
                wl(),
                // Integration
                integration_ssl(),
                integration_sg(),
                // Others
                ping(), 
                help(),
                ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                stripped_dynamic_prefix: Some(|ctx, msg, data| Box::pin(stripped_dynamic_prefix(ctx, msg, data))), 
                case_insensitive_commands: true,
                ..Default::default()
            },
            pre_command: |ctx| {
                Box::pin(async move {
                    ctx.data().logger.log_info(
                        format!("Executing command {} - user: {}({})", ctx.command().qualified_name, ctx.author().name, ctx.author())
                    );
                })
            },
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    wishlist_db,
                    logger
                })
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.
    let client =
        serenity::Client::builder(&token, intents)
                        .framework(framework).await
                        .expect("Err creating client");

    return client;
}

async fn stripped_dynamic_prefix<'a>(
    _: &'a poise::serenity_prelude::Context, 
    msg:&'a Message, 
    _:&'a Data,
) -> Result<Option<(&'a str, &'a str)>, Error> 
{
    if INTEGRATED_COMMANDS.iter().any(|x| msg.content.starts_with(x)) {
        Ok(Some(("", msg.content.as_str())))
    // } else if msg.content.starts_with(".") {
    //     Ok(Some(msg.content.split_at(1)))
    } else {
        Ok(None)
    }
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { .. } => {
            data.logger.log_info("Connected to Discord");
            Ok(())
        }

        // serenity::FullEvent::MessageUpdate { old_if_available, new, event } => {
        //     // TODO: it seems that currently both `new` and `old_if_available` come as None
        //     //       because caching is not correctly used. might look into this in the future
        //     if new.is_some() {
        //         let msg = new.clone().unwrap(); 
        //         return handle(ctx, _framework, data, &msg).await;
        //     } 

        //     if old_if_available.is_some() {
        //         let mut msg = old_if_available.clone().unwrap();
        //         event.apply_to_message(&mut msg);
        //         return handle(ctx, _framework, data, &msg).await;
        //     }

        //     if let Ok(msg) = ctx.http().get_message(event.channel_id, event.id).await {
        //         return handle(ctx, _framework, data, &msg).await;
        //     }

        //     Ok(())
        // }

        serenity::FullEvent::Message { new_message } => {
            handle(ctx, _framework, data, new_message).await
        }
        _ => { Ok(()) }
    }
}

async fn handle(
    ctx: &serenity::Context,
    _framework: poise::FrameworkContext<'_, Data, Error>, 
    data: &Data,
    msg:&Message 
) -> Result<(), Error> {
    match msg.author.id {
        _NORI_USER_ID => {
            if msg.mentions_user_id(_SOFI_USER_ID) {
                wishlist_check_series(ctx, msg, data).await?;
            } else {
                wishlist_check_cards(ctx, msg, data).await?;
            }
        }
        _ => ()
    }

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            ctx.data().logger.log_error(format!("Error in command `{}`: {:?}", ctx.command().name, error,));
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn wishlist_check_series(
    ctx: &serenity::Context, 
    msg: &Message, 
    data: &Data
) -> Result<(), Error> {
    let mut message = MessageBuilder::new();
    message.push("A series from your wishlist is up for grabs!\n");

    let mut send_message = false;
    for line in msg.content.lines() {
        if let Some(series) = parse_series_from_analysis(line) {
            let wishlisted_res = 
                data.wishlist_db.get_users_with_series(series).await;

            if let Err(why) = wishlisted_res {
                data.logger.log_error(format!("wishlist_check_series: Error retrieving wishlisted users for '{series}' : {why:?}"));
                continue;
            } 

            let wishlisted = wishlisted_res.unwrap();

            if wishlisted.len() > 0 
            {
                send_message = true;

                message.push(format!("{series}: \n"));

                for (user, amount) in wishlisted {
                    let user_id = user.parse::<u64>().unwrap();
                    message
                        .push("\t")
                        .mention(&UserId::new(user_id).mention())
                        .push(format!("({amount})\n"));
                    
                    data.logger.log_info(format!("wishlist_check_series: Pinging user `{user_id}` for series `{series}`"));
                }
            }
        }
    }

    if send_message {
        // Build reply to original message 
        let builder = CreateMessage::new().content(message.build()).reference_message(MessageReference::from(msg));
        if let Err(why) = msg.channel_id.send_message(ctx, builder).await {
            data.logger.log_error(format!("Error sending message: {why:?}"));
            return Err(Box::new(why));
        }
    }

    Ok(())
}


async fn wishlist_check_cards(
    ctx: &serenity::Context, 
    msg: &Message, 
    data: &Data
) -> Result<(), Error> {
    let mut message = MessageBuilder::new();
    message.push("A card from your wishlist is dropping!\n");

    let mut wishlist_pings: Vec<(&str, &str, Vec<String>)> = vec![];
    for line in msg.content.lines() {
        if let Some((series, card)) = parse_series_card_from_analysis(line) {
            let wishlisted_res = 
                data.wishlist_db.get_users_with_series_card(series, card).await;

            if let Err(why) = wishlisted_res {
                data.logger.log_error(format!("wishlist_check_cards: Error retrieving wishlisted users for '{card}•{series}' : {why:?}"));
                continue;
            } 

            let wishlisted = wishlisted_res.unwrap();

            if wishlisted.len() > 0 
            {
                wishlist_pings.push((series, card, wishlisted.clone()));
                message.push(format!("{}: ", card));

                for user in wishlisted {
                    let user_id = user.parse::<u64>().unwrap();
                    message.mention(&UserId::new(user_id).mention());
                    data.logger.log_info(format!("wishlist_check_cards: Pinging user `{user_id}` for card `{card}`"));
                }

                message.push("\n");
            }
        }
    }

    if wishlist_pings.is_empty() {
        return Ok(());
    }

    // Build reply to original message 
    let builder = CreateMessage::new().content(message.build()).reference_message(MessageReference::from(msg));

    // Try to send response
    match msg.channel_id.send_message(ctx, builder).await {
        Err(why) => {
            data.logger.log_error(format!("Error sending message: {why:?}"));
            return Err(Box::new(why));
        }
        Ok(reply_msg) => {

            let reaction_one  : ReactionType = ReactionType::Unicode("1️⃣".to_string());
            let reaction_two  : ReactionType = ReactionType::Unicode("2️⃣".to_string());
            let reaction_three: ReactionType = ReactionType::Unicode("3️⃣".to_string());

            if wishlist_pings.len() > 0 { reply_msg.react(ctx, reaction_one.clone()).await?; }
            if wishlist_pings.len() > 1 { reply_msg.react(ctx, reaction_two.clone()).await?; }
            if wishlist_pings.len() > 2 { reply_msg.react(ctx, reaction_three.clone()).await?; }

            while let Some(reaction) = ReactionCollector::new(ctx)
                // only reactions to our reply
                .message_id(reply_msg.id)
                // Timeout when there's no reaction for 60 seconds
                .timeout(std::time::Duration::from_secs(60))
                .await
            {
                let reaction_user = reaction.user(ctx).await?;
                let reaction_user_id = reaction_user.id;

                let opt_ping = match &reaction.emoji { //1️⃣ 2️⃣ 3️⃣
                    ReactionType::Unicode(emoji) if emoji == "1️⃣" => {
                        Some(0)
                    }
                    ReactionType::Unicode(emoji) if emoji == "2️⃣" => {
                        Some(1)
                    }
                    ReactionType::Unicode(emoji) if emoji == "3️⃣" => {
                        Some(2)
                    }
                    _ => None
                }.map(|index| wishlist_pings.get_mut(index).unwrap());

                match opt_ping {
                    Some(ping) => {
                        if ping.2.contains(&reaction_user_id.to_string()) {
                            // remove user that reacted from wishlist_pings internal list if the user is in there
                            ping.2.retain(|user| *user != reaction_user_id.to_string());
        
                            // if it was there, activate wr for him
                            wr_cards( ctx, 
                                Either::Right(reaction.channel_id), 
                                data, 
                                reaction_user_id, 
                                ping.0, 
                                vec![ping.1],
                                None
                            ).await.unwrap();
                        }
                    }
                    None => {}
                }
            }

            reply_msg.delete_reaction_emoji(ctx, reaction_one).await?;
            reply_msg.delete_reaction_emoji(ctx, reaction_two).await?;
            reply_msg.delete_reaction_emoji(ctx, reaction_three).await?;

            return Ok(());
        }
    }
}
