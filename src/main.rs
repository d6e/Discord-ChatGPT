mod commands;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::*;
use dotenv::dotenv;
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "gpt4" => commands::gpt4::run(&command.data.options).await,
                _ => "Unknown command.".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("DISCORD_GUILD_ID")
                .expect("Expected DISCORD_GUILD_ID in environment")
                .parse()
                .expect("DISCORD_GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    commands::gpt4::register(command)
                })
        })
        .await;

        println!("I now have the following guild slash commands: {:#?}", commands);

        // let guild_command = Command::create_global_application_command(&ctx.http, |command| {
        //     commands::gpt4::register(command)
        // })
        // .await;

        // println!("I created the following global slash command: {:#?}", guild_command);
    }
}



#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN environment variable");


    let mut client = serenity::Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating the Discord client");

    if let Err(why) = client.start().await {
        println!("Discord client error: {:?}", why);
    }
}
