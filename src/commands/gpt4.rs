use std::env;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serde::{Deserialize, Serialize};

// Structure for the ChatGPT-4 API response
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    choices: Vec<Answer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Answer {
    text: String,
}


pub async fn run(options: &[CommandDataOption]) -> String {
    let user_query = options
        .get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|value| value.as_str())
        .unwrap_or("");

    match get_gpt4_response(user_query).await {
        Ok(response) => response,
        Err(_) => "Error while processing the request. Please try again later.".to_string(),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("gpt4").description("Query OpenAI ChatGPT4.")
}

async fn get_gpt4_response(prompt: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("OPENAI_KEY").expect("Expected a OPENAI_KEY environment variable");
    
    print!("Querying gpt4 with prompt=\'{}\'", prompt);

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
        "max_tokens": 500,
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