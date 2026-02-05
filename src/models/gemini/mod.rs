use reqwest::Client;
use serde_json::json;
use std::{env, sync::Arc};
mod gemini;
use gemini::{GeminiContent, GeminiPart, GeminiRequest, GeminiResponse};

use crate::app::Chat; // Ensure Part is imported

#[derive(Debug, Clone)]
pub enum Message {
    ApiKeyNotSet,
    RequestError(String),
    ApiResultParsingError(String),
    ApiError(String),
    PromptBlocked(String),
    Response(String),
    EmptyResponse,
}

pub fn convert_to_gemini_request<'a>(history: &'a Arc<Vec<Chat>>) -> GeminiRequest<'a> {
    let contents = history
        .iter()
        .map(|chat| GeminiContent {
            role: &chat.role,
            parts: vec![GeminiPart {
                text: &chat.content,
            }],
        })
        .collect();

    GeminiRequest { contents }
}

pub async fn get_gemini_response(history: Arc<Vec<Chat>>) -> Message {
    let client = Client::new();
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => return Message::ApiKeyNotSet,
    };

    let prompt = convert_to_gemini_request(&history);

    let response: GeminiResponse = match client.post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent")
        .header("x-goog-api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&json!(prompt))
        .send()
        .await {
           Ok(result) => {
               match result.json().await {
                   Ok(result) => result,
                   Err(err) => return Message::ApiResultParsingError(err.to_string())
               }
           },
           Err(err) => return Message::RequestError(err.to_string())
        };

    // 1. Handle API-Level Errors immediately
    if let Some(err) = response.error {
        return Message::ApiError(err.message);
    }

    for candidate in response.candidates.iter().flatten() {
        for rating in candidate.safety_ratings.iter().flatten() {
            if rating.blocked {
                return Message::PromptBlocked(format!(
                    "⚠️ Prompt Blocked by category: {:?}",
                    rating.category
                ));
            }
        }
        // --- Finish Reason ---
        /*
        match candidate.finish_reason.as_ref() {
            Some(FinishReason::Stop) => println!("✅ Response complete"),
            Some(FinishReason::Safety) => println!("⛔ Finished due to Safety"),
            Some(reason) => println!("ℹ️ Finished due to other reason: {:?}", reason),
            None => println!("Finished due to unkown reason"),
        }

        */
        if let Some(part) = candidate.content.parts.iter().last() {
            if let Some(text) = part.text.as_deref() {
                return Message::Response(text.to_string());
            }
        }
    }

    Message::EmptyResponse
}
