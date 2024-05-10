use std::sync::Arc;

use poise::serenity_prelude as serenity;

use ::serenity::all::{CreateMessage, Http, Message, MessageBuilder, MessageReference};
use serenity::all::UserId;
use serenity::prelude::*;

use crate::commands::*;
use crate::component::logger::StdoutLogger;
use crate::util::parse_util::parse_card_from_drop;
use crate::wishlist_db::WishlistDB;
use crate::component::logger::Logger;

const _SOFI_USER_ID:UserId = UserId::new(853629533855809596);
const _SOFU_USER_ID:UserId = UserId::new(950166445034188820);
const _NORI_USER_ID:UserId = UserId::new(742070928111960155);
const _ME_USER_ID :UserId = UserId::new(234822770385485824);

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
        | GatewayIntents::MESSAGE_CONTENT;                      

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                wa(), 
                wr(), 
                wl(),
                ping(), 
                help(),
                ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
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

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { .. } => {
            data.logger.log_info("Connected to Discord");
        }
        serenity::FullEvent::Message { new_message } => {
            match new_message.author.id {
                _NORI_USER_ID => {
                    wishlist_check(ctx, new_message, data).await;
                }
                _ => ()
            }
        }
        _ => {}
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

async fn wishlist_check(ctx: &serenity::Context, msg: &Message, data: &Data) {
    // We assume SOFU/NORI only sends messages about drops
    //   this might change in the future so be mindful
    //   it may become a TODO here

    let mut message = MessageBuilder::new();
    message.push("A card from your wishlist is dropping!\n");

    let mut wishlist_flag = false;

    for line in msg.content.lines() {
        if let Some((series, card)) = parse_card_from_drop(line) {
            let wishlisted_res = 
                data.wishlist_db.get_wishlisted_users(series, card).await;

            if let Err(why) = wishlisted_res {
                data.logger.log_error(format!("Error retrieving wishlisted users for '{card}•{series}' : {why:?}"));
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
                    data.logger.log_info(format!("Pinging user `{user_id}` for card `{card}`"));
                }

                message.push("\n");
            }
        }
    }

    if wishlist_flag {
        send_response(&data.logger, ctx.http(), msg, CreateMessage::new().content(message.build())).await;
    }
}


async fn send_response(logger: &Arc<StdoutLogger>, http: &Http, original_msg: &Message, builder: CreateMessage) {
    // Reply to original message 
    let builder = builder.reference_message(MessageReference::from(original_msg));

    // Try to send
    if let Err(why) = original_msg.channel_id.send_message(http, builder).await {
        logger.log_error(format!("Error sending message: {why:?}"));
    };
}
