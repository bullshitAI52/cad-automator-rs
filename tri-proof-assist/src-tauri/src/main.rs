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
                "title": "三角形内角和",
                "description": "已知：三角形ABC中，∠A=60°，∠B=70°。求：∠C是多少度？",
                "steps": [
                    {"because": "三角形三个角加起来是180°", "therefore": "∠A + ∠B + ∠C = 180°"},
                    {"because": "∠A=60°，∠B=70°", "therefore": "∠C=180°-60°-70°=50°"}
                ]
            }),
            "长方形" => json!({
                "title": "长方形周长",
                "description": "已知：长方形长8厘米，宽5厘米。求：它的周长是多少厘米？",
                "steps": [
                    {"because": "长方形周长=(长+宽)×2", "therefore": "(8+5)×2=26"},
                    {"because": "长8厘米，宽5厘米", "therefore": "周长是26厘米"}
                ]
            }),
            "正方形" => json!({
                "title": "正方形面积",
                "description": "已知：正方形边长是6厘米。求：它的面积是多少平方厘米？",
                "steps": [
                    {"because": "正方形面积=边长×边长", "therefore": "6×6=36"},
                    {"because": "边长是6厘米", "therefore": "面积是36平方厘米"}
                ]
            }),
            "平行四边形" => json!({
                "title": "认识平行四边形",
                "description": "观察下面的图形，它有四条边，对边平行而且相等。说一说：它是什么图形？",
                "steps": [
                    {"because": "它有四条边", "therefore": "它是一个四边形"},
                    {"because": "它的对边平行而且相等", "therefore": "它是一个平行四边形"}
                ]
            }),
            "圆" => json!({
                "title": "圆的周长",
                "description": "已知：一个圆的直径是6厘米。求：它的周长是多少厘米？（π取3.14）",
                "steps": [
                    {"because": "圆的周长=π×直径", "therefore": "3.14×6=18.84"},
                    {"because": "直径是6厘米", "therefore": "周长是18.84厘米"}
                ]
            }),
            _ => json!({
                "title": "认识直角",
                "description": "观察下面的角，它是90°。说一说：它是什么角？",
                "steps": [
                    {"because": "这个角是90°", "therefore": "它是一个直角"},
                    {"because": "直角的两条边互相垂直", "therefore": "可以用三角尺检验"}
                ]
            }),
        };
        return Ok(mock_proof.to_string());
    }

    // 调用DeepSeek API
    let client = reqwest::Client::new();
    let prompt = format!(
        "你是一个小学数学老师，需要为小学生生成一个简单的几何说理练习。难度级别：{}。
        
        请返回一个JSON对象，包含以下字段：
        1. title: 练习标题
        2. description: 题目（已知条件和要解决的问题，用小学生能看懂的话）
        3. steps: 说理步骤数组，每个步骤包含：
           - because: '因为'部分（已知条件或上一步结论）
           - therefore: '所以'部分（推导出的结论）
        
        要求：
        1. 步骤数量：2-4个
        2. 用简单的中文，句子要短
        3. 只用小学数学知识：认识图形、角、周长、面积，不要用全等、勾股定理、根号、相似、圆周角这些初中内容
        4. 数字要简单，适合口算
        5. 适合小学生理解
        
        示例格式：
        {{
          \"title\": \"三角形内角和\",
          \"description\": \"已知：三角形ABC中，∠A=60°，∠B=70°。求：∠C是多少度？\",
          \"steps\": [
            {{\"because\": \"三角形三个角加起来是180°\", \"therefore\": \"∠A + ∠B + ∠C = 180°\"}},
            {{\"because\": \"∠A=60°，∠B=70°\", \"therefore\": \"∠C=180°-60°-70°=50°\"}}
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
