#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Serialize, Deserialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
}

#[tauri::command]
async fn generate_problem(topic: String, difficulty: String, api_key: String) -> Result<String, String> {
    println!("Generating problem for: {} ({})", topic, difficulty);
    
    // Check for API Key (Prioritize UI input, then Env Var)
    let final_api_key = if !api_key.is_empty() {
        api_key
    } else {
        env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "".to_string())
    };

    if final_api_key.is_empty() {
        // Return Mock Data for MVP demonstration
        println!("No API Key found. Returning Mock Data.");
        let mock_problem = json!({
            "title": "Mock Triangle Proof",
            "description": format!("Calculate the hypotenuse of a right-angled triangle with sides 3 and 4. (Topic: {}, Difficulty: {})", topic, difficulty),
            "setup": {
                "shapes": [
                     { "type": "triangle", "points": [0, 100, 0, 0, 100, 100], "stroke": "#00D2FF" }
                ]
            }
        });
        return Ok(mock_problem.to_string());
    }

    // Call DeepSeek API
    let client = reqwest::Client::new();
    let prompt = format!("Create a junior high school geometry problem about {}. Difficulty: {}. Return ONLY valid JSON with 'title', 'description', and 'setup' fields.", topic, difficulty);

    let res = client.post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", final_api_key))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let text = res.text().await.map_err(|e| e.to_string())?;
        // simplistic parsing, assuming API returns raw JSON in content
        // In reality, we'd parse the 'choices[0].message.content' here.
        Ok(text) 
    } else {
        Err(format!("API Request failed: {}", res.status()))
    }
}

#[tauri::command]
async fn save_file(path: String, content: String) -> Result<(), String> {
    println!("Saving file to: {}", path);
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    println!("Reading file from: {}", path);
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_image_as_base64(path: String) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};
    
    println!("Reading image from: {}", path);
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let base64 = general_purpose::STANDARD.encode(&bytes);
    
    // Detect MIME type from extension
    let mime_type = if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/png" // default
    };
    
    Ok(format!("data:{};base64,{}", mime_type, base64))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, generate_problem, save_file, read_file, read_image_as_base64])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
