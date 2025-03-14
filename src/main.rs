use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct Content {
    text: String,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    content: Vec<Content>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ErrorDetail {
    r#type: String,
    message: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ErrorResponse {
    r#type: String,
    error: ErrorDetail,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();

    println!("Chat with Claude (type 'exit' to quit)");
    println!("--------------------------------------");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let request = ChatRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: input.to_string(),
            }],
            max_tokens: 1024,
        };

        match client
            .post("https://api.anthropic.com/v1/messages")
            .headers(headers.clone())
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                let response_text = response.text().await?;

                // Try to parse as error response first
                if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&response_text) {
                    println!("\nAPI Error: {}\n", error_response.error.message);
                    continue;
                }

                // If it's not an error, try to parse as success response
                match serde_json::from_str::<ChatResponse>(&response_text) {
                    Ok(chat_response) => {
                        if let Some(content) = chat_response.content.first() {
                            println!("\nClaude: {}\n", content.text);
                        }
                    }
                    Err(e) => {
                        println!("\nError parsing response: {}", e);
                        println!("Raw response: {}", response_text);
                    }
                }
            }
            Err(e) => println!("\nError sending request: {}", e),
        }
    }

    Ok(())
}
