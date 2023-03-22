use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;

// Structure for the ChatGPT-4 API response
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    choices: Vec<Answer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Answer {
    text: String,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "chatgpt4" => handle_chatgpt4_command(command.clone(), &ctx).await,
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
}

async fn handle_chatgpt4_command(
    command: ApplicationCommandInteraction,
    ctx: &Context,
) -> String {
    let user_query = command
        .data
        .options
        .get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|value| value.as_str())
        .unwrap_or("");

    match get_gpt4_response(user_query).await {
        Ok(response) => response,
        Err(_) => "Error while processing the request. Please try again later.".to_string(),
    }
}

async fn get_gpt4_response(prompt: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("OPENAI_KEY").expect("Expected a OPENAI_KEY environment variable");
    
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/engines/davinci-codex/completions";
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );

    let body = serde_json::json!({
        "prompt": prompt,
        "max_tokens": 50,
        "n": 1,
        "stop": ["\n"],
    });

    let res = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    let api_response: ApiResponse = res.json().await?;

    if let Some(answer) = api_response.choices.get(0) {
        Ok(answer.text.trim().to_string())
    } else {
        Ok("No response from the GPT-4 API.".to_string())
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
