use std::cmp::min;

use poise::serenity_prelude as serenity;
use poise::samples::HelpConfiguration;
use poise::CreateReply;
use rand::Rng;
use ::serenity::all::{ChannelId, ComponentInteractionCollector, CreateEmbedFooter, CreateMessage, EditMessage, UserId};
use serenity::all::MessageBuilder;
use serenity::all::{Message, User};

use crate::components::logger::Logger;
use crate::util::either::Either;
use crate::util::parse_util::parse_series_cards;
use crate::bot::{Context, Data, Error};

// ##############################
// ##############################  WISHLIST ADD
// ##############################

/// Adds all selected cards from a series to your wishlist.
/// Will not add duplicates.
#[poise::command(prefix_command, rename = "wa")]
pub async fn command_wa (
    ctx: Context<'_>,
    #[description = "<series> || <card name> (, <card name>)*"]
    #[rest] command: String,
) -> Result<(), Error> 
{
    match parse_series_cards(&command) {
        None => { 
            ctx.reply("Incorrect argument format. Check `.help wa`").await.unwrap();
            Ok(())
        },
        Some((series, card_names)) => {
           wa(ctx.serenity_context()
             , &ctx.http().get_message(ctx.channel_id(), ctx.id().into()).await.unwrap()
             , ctx.data()
             , ctx.author().id
             , series
             , card_names
             , None
            ).await.map(|_| ())
        },
    }
}

pub async fn wa (
    ctx: &serenity::Context, 
    msg: &Message, 
    data: &Data, 
    user_id: UserId, 
    series: &str, 
    card_names: Vec<&str>,
    prev_response: Option<(Message, i32)>
) -> Result<(Message, i32), Error> 
{
    let mut message = MessageBuilder::new();

    let res = 
        data.wishlist_db.add_all_to_wishlist(&user_id.to_string(), series, card_names).await;

    match res {
        Ok(added_cards_count) => {
            match prev_response {
                Some((mut prev_msg, prev_added_count)) => {
                    let total = added_cards_count + prev_added_count;
                    message.push(format!("Added {total} card(s) from `{series}` to your wishlist!"));
                    prev_msg.edit(ctx, EditMessage::new().content(message.build())).await.unwrap();
                    return Ok((prev_msg, total));
                },
                None => {
                    message.push(format!("Added {added_cards_count} card(s) from `{series}` to your wishlist!"));
                    let response_msg = msg.reply_ping(ctx, message.build()).await.unwrap();
                    return Ok((response_msg, added_cards_count));
                }
            }
        },
        Err(err) =>  {
            data.logger.log_error(format!(".wa | {}", err.to_string()));
            message.push(format!("Something went wrong adding cards to your wishlist."));
            return Err(err);
        }
    };
}

// ##############################
// ##############################  WISHLIST REMOVE
// ##############################

/// Removes all selected cards from a series OR an entire series (if no cards are specified) from your wishlist.
/// Will only remove cards already in your wishlist.
#[poise::command(prefix_command, rename = "wr")]
pub async fn command_wr(
    ctx: Context<'_>,
    #[description = "<series> ( || <card name> (, <card name>)* )?"]
    #[rest] command: String,
) -> Result<(), Error> 
{
    if !command.contains("||") {
        // Delete entire series
        wr_series( ctx.serenity_context()
                 , &ctx.http().get_message(ctx.channel_id(), ctx.id().into()).await.unwrap()
                 , ctx.data()
                 , ctx.author().id
                 , command.as_str()
                 ).await;       
    } else {
        // Delete selected cards from series
        match parse_series_cards(&command) {
            None => { ctx.reply("Incorrect argument format. Check `.help wr`").await.unwrap(); },
            Some((series, card_names)) => {
                wr_cards( ctx.serenity_context()
                        , Either::Left(&ctx.http().get_message(ctx.channel_id(), ctx.id().into()).await.unwrap())
                        , ctx.data()
                        , ctx.author().id
                        , series
                        , card_names
                        , None
                        ).await.unwrap();
            }
        }
    }

    Ok(())
}

pub async fn wr_cards (
    ctx: &serenity::Context, 
    user_msg: Either<&Message, ChannelId>, 
    data: &Data, 
    user_id: UserId, 
    series: &str, 
    card_names: Vec<&str>,
    prev_response: Option<(Message, i32)>
) -> Result<(Message, i32), Error> 
{
    let mut message = MessageBuilder::new();

    let res = 
        data.wishlist_db.remove_all_from_wishlist(&user_id.to_string(), series, card_names).await; 

    match res {
        Ok((amount_removed, amount_left)) => {
            match prev_response {
                Some((mut prev_msg, prev_removed_count)) => {
                    let total = prev_removed_count + amount_removed;

                    message.push(format!("Removed {total} card(s) from your wishlist!"));
                    prev_msg.edit(ctx, EditMessage::new().content(message.build())).await.unwrap();

                    return Ok((prev_msg, total));
                },
                None => {
                    if amount_left > 0 {
                        message.push(format!("Removed {amount_removed} card(s) from your wishlist! ({amount_left} card(s) left)"));
                    } else {
                        message.push(format!("Removed {amount_removed} card(s) from your wishlist! No more cards left!"));
                    }

                    let response = match user_msg {
                        Either::Left(msg) => msg.reply_ping(ctx, message.build()).await.unwrap(),
                        Either::Right(channel_id) => {
                            message.user(user_id);
                            let builder = CreateMessage::new().content(message.build());
                            ctx.http.send_message(channel_id, vec![], &builder).await.unwrap()
                        }
                    };

                    return Ok((response, amount_removed));
                }
            }
        },
        Err(err) => {
            data.logger.log_error(err.to_string());
            message.push(format!("Something went wrong removing cards from your wishlist."));
            match user_msg {
                Either::Left(msg) => msg.reply_ping(ctx, message.build()).await.unwrap(),
                Either::Right(channel_id) => {
                    message.push(" ");
                    message.user(user_id);
                    let builder = CreateMessage::new().content(message.build());
                    ctx.http.send_message(channel_id, vec![], &builder).await.unwrap()
                }
            };

            return Err(err);
        }
    };
}

pub async fn wr_series(
    ctx: &serenity::Context, 
    msg: &Message, 
    data: &Data, 
    user_id: UserId, 
    series: &str
) {
    let res = 
    data.wishlist_db.remove_series_from_wishlist(&user_id.to_string(), series).await; 
    
    let mut message = MessageBuilder::new();
    match res {
        Ok(amount) => message.push(format!("Removed series `{series}` with {amount} card(s) from your wishlist!")),
        Err(err) => {
            data.logger.log_error(err.to_string());
            message.push(format!("Something went wrong removing a series from your wishlist."))
        }
    };

    msg.reply_ping(ctx, message.build()).await.unwrap();
}

// ##############################
// ##############################  WISHLIST LIST
// ##############################

/// List all series, or cards from a series in your wishlist.
#[poise::command(prefix_command)]
pub async fn wl(
    ctx: Context<'_>,
    #[description = "Target user"]
    user: Option<User>,
    #[description = "Series name"]
    #[rest] content: Option<String>,
) -> Result<(), Error> 
{
    let user_id = user.map(|user| user.id).unwrap_or(ctx.author().id);

    let (pages, total_count) = match content {
        None => {
            let wishlisted_series = ctx.data().wishlist_db.get_user_wishlisted_series(&user_id.to_string()).await;
            let total_size = wishlisted_series.len();

            let wishlisted_series_chunks = wishlisted_series.chunks(10);

            let mut series_pages = Vec::with_capacity(wishlisted_series_chunks.len());
            for series_chunk in wishlisted_series_chunks {
                let mut series_page = Vec::with_capacity(series_chunk.len());

                for series in series_chunk {
                    let count = ctx.data().wishlist_db.get_user_wishlisted_cards_count(&user_id.to_string(), series).await;
                    series_page.push(format!("{series} ({count})"));
                }

                series_pages.push(series_page.join("\n"))
            };

            (series_pages, total_size)
        },
        Some(series) => {
            let mut wishlisted_cards = ctx.data().wishlist_db.get_user_wishlisted_cards(&user_id.to_string(), &series).await;
            let total_size = wishlisted_cards.len();

            ( wishlisted_cards.chunks_mut(10)
                .map(|chunk| {
                    chunk.iter_mut().for_each(|s| s.truncate(32));
                    chunk.join("\n")
                })
                .collect()
            , total_size
            )
        }
    };

    paginate(ctx, pages, total_count).await?;

    Ok(())
}


// ##############################
// ##############################  PING
// ##############################

const PONG_GIF_LINKS: [&'static str; 6]  = [
    "https://tenor.com/view/pong-gif-26462133",
    "https://tenor.com/view/bombardierul-pazitor-pong-maca-pong-gif-25389982",
    "https://tenor.com/view/get-ponged-pong-lol-troll-gif-20311938",
    "https://tenor.com/view/ping-pong-gif-26618047",
    "https://tenor.com/view/ping-pong-avast-alone-waiting-gif-8485903",
    "https://tenor.com/view/pingpong-ping-pong-pong-pro-pong-table-tennis-gif-12226811345916051594"
];

/// Pong!
#[poise::command(prefix_command, category = "Others")]
pub async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> 
{
    let random_n = rand::thread_rng().gen_range(0..PONG_GIF_LINKS.len());

    ctx.reply(PONG_GIF_LINKS[random_n]).await?;

    Ok(())
}

// ##############################
// ##############################  HELP
// ##############################

#[poise::command(prefix_command, track_edits, category = "Utility")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"]
    #[rest]
    mut command: Option<String>,
) -> Result<(), Error> {
    // This makes it possible to just make `help` a subcommand of any command
    // `/fruit help` turns into `/help fruit`
    // `/fruit help apple` turns into `/help fruit apple`
    if ctx.invoked_command_name() != "help" {
        command = match command {
            Some(c) => Some(format!("{} {}", ctx.invoked_command_name(), c)),
            None => Some(ctx.invoked_command_name().to_string()),
        };
    }
    let extra_text_at_bottom = "\
Type `.help command` for more info on a command.
You can edit your `.help` message to the bot and the bot will edit its response.";

    let config = HelpConfiguration {
        show_subcommands: true,
        show_context_menu_commands: true,
        ephemeral: true,
        extra_text_at_bottom,

        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}























pub async fn paginate (
    ctx: Context<'_>,
    pages: Vec<String>,
    total_size: usize,
) -> Result<(), serenity::Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let reply = {
        let components = if pages.len() == 0 {
            vec![]
        } else {
            vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                serenity::CreateButton::new(&next_button_id).emoji('▶'),
            ])]
        };

        CreateReply::default()
            .embed(
                serenity::CreateEmbed::default()
                    .description(pages.get(0).unwrap_or(&"Nothing to show".to_string()))
                    .footer(CreateEmbedFooter::new(format!("Page {}/{} (Total {})", min(pages.len(), 1), pages.len(), total_size)))
                )
            .components(components)
    };

    ctx.send(reply).await?;

    if pages.len() == 0 {
        return Ok(());
    }

    // Loop through incoming interactions with the navigation buttons
    let mut current_page = 0;
    while let Some(press) = ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Paginate only works for owner
        .author_id(ctx.author().id)
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(
                            serenity::CreateEmbed::new()
                                .description(pages.get(current_page).unwrap())
                                .footer(CreateEmbedFooter::new(format!("Page {}/{} (Total {})", current_page + 1, pages.len(), total_size)))
                            ),
                ),
            )
            .await?;
    }

    Ok(())
}