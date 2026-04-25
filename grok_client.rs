use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GROK_API_URL: &str = "https://api.x.ai/v1/chat/completions";

#[derive(Serialize)]
struct GrokRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
struct GrokResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

const LEGAL_SYSTEM_PROMPT: &str = r#"You are a qualified legal AI assistant for contract analysis.
Your task is to find risks in the document and provide recommendations.

Response format (strictly JSON):
{
    "status": "success" or "error",
    "risk_score": number from 0 to 100,
    "findings": ["finding 1", "finding 2", ...],
    "ai_suggestion": "detailed recommendation in Russian"
}

Common risks to look for:
- Missing essential contract terms
- Unfavorable payment terms (over 30 days)
- Lack of liability clauses
- Incorrect company details
- VAT/tax risks
- Missing force majeure clause (especially relevant for 2026 sanctions)
- Vague wording
- Unfavorable jurisdiction for the client
- Currency risks
- Missing confidentiality clause

Reply ONLY with valid JSON without markdown formatting."#;

pub fn analyze_document_sync(api_key: &str, document_text: &str, depth: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let max_tokens = if depth == "deep" { 2000 } else { 1000 };
    let temperature = if depth == "deep" { 0.4 } else { 0.2 };

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: LEGAL_SYSTEM_PROMPT.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!(
                "Analyze this contract (depth: {}):\n\n{}", 
                depth, 
                document_text
            ),
        },
    ];

    let request = GrokRequest {
        model: "grok-beta".to_string(),
        messages,
        temperature,
        max_tokens,
    };

    let response = client
        .post(GROK_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Grok API error: {}", response.status()));
    }

    let grok_response: GrokResponse = response
        .json()
        .map_err(|e| format!("Parse error: {}", e))?;

    if let Some(choice) = grok_response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("Empty response from Grok".to_string())
    }
}
