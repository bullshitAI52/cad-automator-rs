#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde_json::json;
use std::env;

#[tauri::command]
async fn generate_problem(
    topic: String,
    difficulty: String,
    api_key: String,
) -> Result<String, String> {
    println!("生成几何证明思路: {} ({})", topic, difficulty);

    // 检查API Key
    let final_api_key = if !api_key.is_empty() {
        api_key
    } else {
        env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "".to_string())
    };

    if final_api_key.is_empty() {
        // 返回模拟数据
        println!("未找到API Key，返回模拟数据");
        let mock_proof = match topic.as_str() {
            "三角形" => json!({
                "title": "三角形全等证明示例",
                "description": "已知：在△ABC和△DEF中，AB=DE，∠A=∠D，AC=DF。证明：△ABC≅△DEF。",
                "steps": [
                    {"because": "AB = DE", "therefore": "对应边相等"},
                    {"because": "∠A = ∠D", "therefore": "对应角相等"},
                    {"because": "AC = DF", "therefore": "对应边相等"},
                    {"because": "两边及其夹角对应相等", "therefore": "△ABC ≅ △DEF (SAS全等)"}
                ]
            }),
            "四边形" => json!({
                "title": "平行四边形性质证明",
                "description": "已知：四边形ABCD中，AB∥CD且AB=CD。证明：ABCD是平行四边形。",
                "steps": [
                    {"because": "AB ∥ CD", "therefore": "一组对边平行"},
                    {"because": "AB = CD", "therefore": "这组对边相等"},
                    {"because": "一组对边平行且相等", "therefore": "四边形ABCD是平行四边形"}
                ]
            }),
            "圆" => json!({
                "title": "圆周角定理",
                "description": "已知：在⊙O中，弧AB所对的圆周角是∠ACB，圆心角是∠AOB。证明：∠ACB = 1/2∠AOB。",
                "steps": [
                    {"because": "连接OC", "therefore": "构造两个三角形"},
                    {"because": "OA=OC=OB", "therefore": "都是半径，相等"},
                    {"because": "等腰三角形底角相等", "therefore": "可以推导角度关系"},
                    {"because": "三角形内角和180°", "therefore": "∠ACB = 1/2∠AOB"}
                ]
            }),
            _ => json!({
                "title": "几何证明示例",
                "description": format!("这是一个关于{}的证明示例。请按照'因为→所以'的格式填写证明步骤。", topic),
                "steps": [
                    {"because": "填写已知条件", "therefore": "填写推导结论"},
                    {"because": "使用几何定理", "therefore": "得到中间结论"},
                    {"because": "综合以上结论", "therefore": "证明完成"}
                ]
            }),
        };
        return Ok(mock_proof.to_string());
    }

    // 调用DeepSeek API
    let client = reqwest::Client::new();
    let prompt = format!(
        "你是一个几何老师，需要为初中生生成一个关于{}的几何证明思路。难度级别：{}。
        
        请返回一个JSON对象，包含以下字段：
        1. title: 证明标题
        2. description: 证明描述（已知条件和要证明的结论）
        3. steps: 证明步骤数组，每个步骤包含：
           - because: '因为'部分（已知条件或上一步结论）
           - therefore: '所以'部分（推导出的结论）
        
        要求：
        1. 步骤数量：3-5个
        2. 使用中文
        3. 包含几何符号如∠、△、≅、⊥、∥等
        4. 适合初中生理解
        
        示例格式：
        {{
          \"title\": \"三角形全等证明\",
          \"description\": \"已知：AB=DE, ∠A=∠D, AC=DF。证明：△ABC≅△DEF。\",
          \"steps\": [
            {{\"because\": \"AB = DE\", \"therefore\": \"对应边相等\"}},
            {{\"because\": \"∠A = ∠D\", \"therefore\": \"对应角相等\"}}
          ]
        }}",
        topic, difficulty
    );

    let res = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", final_api_key))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 1000
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let response_text = res.text().await.map_err(|e| e.to_string())?;

        // 解析API响应
        let response_json: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| format!("解析API响应失败: {}", e))?;

        // 提取AI返回的内容
        let ai_content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("API响应格式错误")?;

        // 尝试解析AI返回的JSON
        match serde_json::from_str::<serde_json::Value>(ai_content) {
            Ok(parsed_content) => {
                // 如果AI返回了有效的JSON，直接使用
                Ok(parsed_content.to_string())
            }
            Err(_) => {
                // 如果AI没有返回有效JSON，包装成标准格式
                let wrapped_response = json!({
                    "title": format!("关于{}的证明", topic),
                    "description": ai_content,
                    "steps": [
                        {"because": "请根据AI生成的思路", "therefore": "填写具体的证明步骤"}
                    ]
                });
                Ok(wrapped_response.to_string())
            }
        }
    } else {
        Err(format!("API请求失败: {}", res.status()))
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
    use base64::{engine::general_purpose, Engine as _};

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

#[tauri::command]
async fn save_image_from_base64(path: String, base64_data: String) -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::io::Write;

    println!("Saving image to: {}", path);

    // Remove data URL prefix if present (e.g., "data:image/jpeg;base64,")
    let base64_clean = if let Some(index) = base64_data.find(',') {
        &base64_data[index + 1..]
    } else {
        &base64_data
    };

    let bytes = general_purpose::STANDARD
        .decode(base64_clean)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            generate_problem,
            save_file,
            read_file,
            read_image_as_base64,
            save_image_from_base64
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
