mod commands;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::*;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{StatusCode};
use tokio::signal::ctrl_c;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "gpt4" => {

                    // Clone context and channel_id to be used in the spawned task
                    let ctx_clone = ctx.clone();
                    let channel_id = command.channel_id;
                    let command_clone = command.clone();

                    // Spawn a new task to send the delayed response
                    tokio::spawn(async move {
                        // Simulate a delay (e.g., a long-running task)
                        let cmd_response = commands::gpt4::run(&command_clone.data.options).await;
                        // Send the delayed response
                        if let Err(why) = channel_id.say(&ctx_clone.http, cmd_response).await {
                            println!("Error on replying to discord: {:?}", why)
                        };
                    });

                    // Respond with the prompt quickly to avoid timing out with discord
                    command.data.options
                        .get(0)
                        .and_then(|opt| opt.value.as_ref())
                        .and_then(|value| value.as_str())
                        .unwrap()
                        .to_string()
                }
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
            commands.create_application_command(|command| commands::gpt4::register(command))
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

async fn health_check(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // If the service is healthy, return a 200 OK status
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Service is healthy"))
        .unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() {
    println!("Started!");
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN environment variable");

    let mut client = serenity::Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating the Discord client");

    let shard_manager = client.shard_manager.clone();

    // Start the health check server
    let health_check_addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(health_check)) }
    });

    let health_check_server = Server::bind(&health_check_addr).serve(make_svc);

    // Spawn the health check server as a separate task
    let health_check_task = tokio::spawn(async move {
        health_check_server.await.unwrap();
    });


    tokio::select! {
        res = client.start() => {
            if let Err(why) = res {
                println!("Client error: {:?}", why);
            }
        },
        _ = ctrl_c() => {
            println!("Ctrl+C received, shutting down...");
            shard_manager.lock().await.shutdown_all().await;
        },
        _ = health_check_task => {
            println!("Health check task finished");
        }
    }
}
