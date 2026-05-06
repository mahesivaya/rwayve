use crate::email::oauth::HTTP_CLIENT;
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

#[derive(Deserialize)]
pub struct ChatTurn {
    /// "user" or "model"
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    /// Full conversation history including the latest user message at the end.
    /// Caller is responsible for ordering and trimming for token limits.
    pub messages: Vec<ChatTurn>,
}

/// POST /api/ai/chat — proxies to Google's Generative Language API
/// (Gemini). The API key lives in GEMINI_API_KEY on the server and is
/// never exposed to the browser. Auth is JWT-gated like the rest of /api.
#[post("/ai/chat")]
pub async fn ai_chat(req: HttpRequest, data: web::Json<ChatRequest>) -> impl Responder {
    if get_user_id_from_request(&req).is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            println!("ai_chat: GEMINI_API_KEY missing");
            return HttpResponse::InternalServerError()
                .body("AI not configured (GEMINI_API_KEY missing)");
        }
    };

    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());

    // Map our {role, content} turns to Gemini's {role, parts:[{text}]} shape.
    // Gemini accepts roles "user" and "model"; anything else gets coerced
    // to "user" so a wrong client doesn't 400 the whole request.
    let contents: Vec<Value> = data
        .messages
        .iter()
        .filter(|m| !m.content.trim().is_empty())
        .map(|m| {
            let role = if m.role == "model" { "model" } else { "user" };
            serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }],
            })
        })
        .collect();

    if contents.is_empty() {
        return HttpResponse::BadRequest().body("Empty conversation");
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let body = serde_json::json!({ "contents": contents });

    let res = HTTP_CLIENT.post(&url).json(&body).send().await;

    let res = match res {
        Ok(r) => r,
        Err(e) => {
            println!("ai_chat upstream error: {:?}", e);
            return HttpResponse::BadGateway().body("Upstream error");
        }
    };

    let status = res.status();
    let payload: Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            println!("ai_chat parse error: {:?}", e);
            return HttpResponse::BadGateway().body("Bad upstream response");
        }
    };

    if !status.is_success() {
        let msg = payload["error"]["message"]
            .as_str()
            .unwrap_or("Upstream error")
            .to_string();
        println!("ai_chat upstream {}: {}", status, msg);
        return HttpResponse::BadGateway().body(msg);
    }

    // Stitch together every text part of the first candidate. Gemini may
    // split a single response across multiple parts (e.g. when grounding
    // metadata is mixed in).
    let reply = payload["candidates"][0]["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({ "reply": reply }))
}
