use std::env;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serde::{Deserialize, Serialize};

// Structure for the ChatGPT-4 API response
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    id: String,
    object: String,
    created: u64,
    choices: Vec<Answer>,
    usage: Usage
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32
}

#[derive(Debug, Serialize, Deserialize)]
struct Answer {
    index: u32,
    message: Message,
    finish_reason: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

pub async fn run(options: &[CommandDataOption]) -> String {
    let user_query = options
        .get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|value| value.as_str())
        .unwrap_or("");

    match get_gpt4_response(user_query).await {
        Ok(response) => response,
        Err(e) => format!("Error while processing the request: {e}"),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("gpt4")
        .description("Query OpenAI ChatGPT4.")
        .create_option(|option|{
            option
                .name("prompt")
                .description("The prompt to pass to gpt")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

async fn get_gpt4_response(prompt: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("OPENAI_KEY").expect("Expected a OPENAI_KEY environment variable");
    
    println!("Querying gpt4 with prompt: {prompt}");

    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
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
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}],
        // "temperature": 0.7,
        // "max_tokens": 5,
        // "top_p": 1,
        // "n": 1,
        // "stop": ["\n"],
    });

    let res = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    let code = res.status();
    if !code.is_success(){
        let status_code = code.as_str().to_string();
        println!("status_code: {status_code}");
        return Ok(status_code)
    }

    let api_response: ApiResponse = res.json().await?;

    if let Some(answer) = api_response.choices.get(0) {
        let ans = answer.message.content.trim().to_string();
        println!("response: {ans}");
        Ok(ans)
    } else {
        Ok("No response from the GPT-4 API.".to_string())
    }
}