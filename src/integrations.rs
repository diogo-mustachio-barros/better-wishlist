
use std::time::Duration;

use ::serenity::all::{Message, MessageCollector, ReactionType};
use serenity::all::ReactionCollector;
use poise::serenity_prelude as serenity;

use crate::commands::{wa, wr_cards};
use crate::components::logger::Logger;
use crate::bot::{Context, Error, _SOFI_USER_ID};
use crate::util::either::Either;
use crate::util::parse_util::{parse_card_from_series_lookup, parse_series_from_embed_description, parse_series_from_give_command};

pub const INTEGRATED_COMMANDS: [&str; 2] = [
    INTEGRATION_SOFI_SSL,
    INTEGRATION_SOFI_SG
];

// ##############################
// ##############################  SOFI SSL 
// ##############################

pub const INTEGRATION_SOFI_SSL: &str = "ssl";

// Adds unnowned cards or removes owned cards from SOFI's show collection whenever the user
//   reacts with either the checkmark or the cross emoji (respectively)
#[poise::command(prefix_command, rename = "ssl", hide_in_help = true)]
pub async fn integration_ssl (
    ctx: Context<'_>,
    #[rest] _full_command: Option<String>,
) -> Result<(), Error> 
{
    let msg_id = ctx.id().clone();

    if let Some(first_reply) = MessageCollector::new(ctx)
        .author_id(_SOFI_USER_ID)
        .channel_id(ctx.channel_id())
        .filter(move |msg| msg.clone().message_reference.is_some_and(|msg_ref| msg_ref.message_id.is_some_and(|id| id.get() == msg_id)))
        .timeout(Duration::from_secs(10))
        .await
    {
        let first_msg = match ctx.http().get_message(ctx.channel_id(), ctx.id().into()).await {
            Ok(msg) => msg,
            Err(err) => {
                ctx.data().logger.log_error(format!("integration_ssl: unable to retrieve {}'s message:\n{err:?}", ctx.author()));
                return Err(Box::new(err));
            }
        };

        let sofi_msg = match ctx.http().get_message(first_reply.channel_id, first_reply.id).await {
            Ok(msg) => msg,
            Err(err) => {
                ctx.data().logger.log_error(format!("integration_ssl: unable to retrieve SOFI's reply to {}:\n{err:?}", ctx.author()));
                return Err(Box::new(err));
            }
        };

        sofi_msg.react(ctx.http(), ReactionType::Unicode("✅".to_string())).await.unwrap();
        sofi_msg.react(ctx.http(), ReactionType::Unicode("❌".to_string())).await.unwrap();

        let mut wa_response_msg: Option<(Message, i32)> = None;
        let mut wr_response_msg: Option<(Message, i32)> = None;

        while let Some(reaction) = ReactionCollector::new(ctx)
            // only the target user's reaction activates the integration
            .author_id(ctx.author().id)
            // only reactions to SOFI's reply
            .message_id(sofi_msg.id)
            // Timeout when there's no reaction for 60 seconds
            .timeout(std::time::Duration::from_secs(60))
            .await
        {
            if !is_series_lookup(&sofi_msg) {
                continue;
            }

            let embed = sofi_msg.embeds.get(0).unwrap();
            let description = embed.description.clone().unwrap();
            let series = parse_series_from_embed_description(description.as_str()).unwrap();
            let cards: Vec<(bool, &str)> = 
                embed.fields.get(0).unwrap().value.split("\n").into_iter().map(parse_card_from_series_lookup).map(Option::unwrap).collect();

            match &reaction.emoji {
                ReactionType::Unicode(emoji) if emoji == "✅" => {
                    reaction.delete(ctx.http()).await?;
                    
                    let card_names = cards.iter()
                    .filter(|card| !card.0)
                    .map(|(_, card)| *card)
                    .collect();
    
                    let response = wa( ctx.serenity_context(), 
                        &first_msg, 
                        ctx.data(), 
                        ctx.author().id, 
                        series, 
                        card_names,
                        wa_response_msg
                    ).await.unwrap();

                    wa_response_msg = Some(response);
                }
                ReactionType::Unicode(emoji) if emoji == "❌" => {
                    reaction.delete(ctx.http()).await?;
                    
                    let card_names = cards.iter()
                    .filter(|card| card.0)
                    .map(|(_, card)| *card)
                    .collect();
                    
                    let response = wr_cards( ctx.serenity_context(), 
                              Either::Left(&first_msg), 
                              ctx.data(), 
                              ctx.author().id, 
                              series, 
                              card_names,
                              wr_response_msg
                    ).await.unwrap();

                    wr_response_msg = Some(response);
                }
                _ => ()
            }
        }
    }

    Ok(())
}

fn is_series_lookup(message: &Message) -> bool {
    message.embeds.get(0).is_some_and(|embed| 
        embed.title.clone().is_some_and(|title| title == "SOFI: SERIES LOOKUP") 
        && embed.fields.len() > 0
    )
}

// ##############################
// ##############################  SOFI SG 
// ##############################

pub const INTEGRATION_SOFI_SG: &str = "sg";

// Adds unnowned cards or removes owned cards from SOFI's show collection whenever the user
//   reacts with either the checkmark or the cross emoji (respectively)
#[poise::command(prefix_command, rename = "sg", hide_in_help = true)]
pub async fn integration_sg (
    ctx: Context<'_>,
    #[rest] _full_command: Option<String>,
) -> Result<(), Error> 
{
    let msg_id = ctx.id().clone();

    if let Some(first_reply) = MessageCollector::new(ctx)
        .author_id(_SOFI_USER_ID)
        .channel_id(ctx.channel_id())
        .filter(move |msg| msg.clone().message_reference.is_some_and(|msg_ref| msg_ref.message_id.is_some_and(|id| id.get() == msg_id)))
        .timeout(Duration::from_secs(10))
        .await
    {
        let first_msg = match ctx.http().get_message(ctx.channel_id(), ctx.id().into()).await {
            Ok(msg) => msg,
            Err(err) => {
                ctx.data().logger.log_error(format!("integration_sg: unable to retrieve {}'s message:\n{err:?}", ctx.author()));
                return Err(Box::new(err));
            }
        };

        let sofi_msg = match ctx.http().get_message(first_reply.channel_id, first_reply.id).await {
            Ok(msg) => msg,
            Err(err) => {
                ctx.data().logger.log_error(format!("integration_sg: unable to retrieve SOFI's reply to {}:\n{err:?}", ctx.author()));
                return Err(Box::new(err));
            }
        };

        let target_user = first_msg.mentions.get(0).unwrap();
        
        let embed = sofi_msg.embeds.get(0).unwrap();
        let description = embed.description.clone().unwrap();
        let (series, card) = parse_series_from_give_command(description.as_str()).unwrap();
        let has_card = ctx.data().wishlist_db.user_has_card(target_user.id.to_string().as_str(), series, card).await;
        
        if has_card {
            sofi_msg.react(ctx.http(), ReactionType::Unicode("❌".to_string())).await.unwrap();

            while let Some(reaction) = ReactionCollector::new(ctx)
                // only the target user's reaction activates the integration
                .author_id(target_user.id)
                // only reactions to SOFI's reply
                .message_id(sofi_msg.id)
                // Timeout when there's no reaction for 60 seconds
                .timeout(std::time::Duration::from_secs(60))
                .await
            {
                match &reaction.emoji {
                    ReactionType::Unicode(emoji) if emoji == "❌" => {
                        reaction.delete_all(ctx.http()).await?;
                        
                        wr_cards( ctx.serenity_context(), 
                                  Either::Right(ctx.channel_id()), 
                                  ctx.data(), 
                                  target_user.id, 
                                  series, 
                                  vec![card],
                                  None
                        ).await.unwrap();
                    }
                    _ => ()
                }
            }
        }
    }

    Ok(())
}