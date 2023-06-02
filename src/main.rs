//! Possum Bot, possum photos Discord bot
//! Copyright (C) 2023  Valentine Briese
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU Affero General Public License as published
//! by the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU Affero General Public License for more details.
//!
//! You should have received a copy of the GNU Affero General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Attachment, AttachmentType, GuildId};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use sqlx::{Executor, PgPool, Row};
use std::borrow::Cow;

struct Data {
    pool: PgPool,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Responds with a random possum picture.
#[poise::command(
    slash_command,
    name_localized("en-GB", "opossum"),
    description_localized("en-GB", "Responds with a random opossum picture.")
)]
async fn possum(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let image = ctx
        .data()
        .pool
        .fetch_one("SELECT id, image_data, attribution FROM approved ORDER BY RANDOM() LIMIT 1;")
        .await?;

    ctx.send(|reply| {
        reply
            .content(image.get::<String, &str>("attribution"))
            .attachment(AttachmentType::Bytes {
                data: Cow::from(image.get::<&[u8], &str>("image_data")),
                filename: format!("possum-bot-image-{}", image.get::<String, &str>("id")),
            })
    })
    .await?;

    Ok(())
}

/// Submits an image for review to be added to the images the bot can return.
#[poise::command(slash_command)]
async fn submit(
    ctx: Context<'_>,
    #[description = "The image to submit."] image: Attachment,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    ctx.say(image.content_type.unwrap_or("None".to_string()))
        .await?;

    Ok(())
}

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttlePoise<Data, Error> {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // let image_hex = include_bytes!("Didelphis_virginiana_with_young.JPG")
    //     .map(|byte| format!("{:02X?}", byte))
    //     .join("");
    //
    // debug!(image_hex);
    //
    // pool.execute(format!(
    //     "INSERT INTO approved (image_data, attribution) VALUES ('\\x{}', 'Specialjake, CC BY-SA 3.0 <https://creativecommons.org/licenses/by-sa/3.0>, via Wikimedia Commons');",
    //     image_hex,
    // ).as_str())
    // .await
    // .map_err(shuttle_runtime::CustomError::new)?;

    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let testing_guild_id = secret_store
        .get("TESTING_GUILD_ID")
        .context("'TESTING_GUILD_ID' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![possum()],
            allowed_mentions: None,
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId(testing_guild_id.parse()?),
                )
                .await?;

                Ok(Data { pool })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
