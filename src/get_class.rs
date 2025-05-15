use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct CourseInfo {
    pub classroom_name: String,
    pub attendance_state: i32,
    pub course_plan_id: String,
    pub attendance_id: String,
}

pub  async  fn get_class(cookie: &str) -> Result<Option<Vec<CourseInfo>>, reqwest::Error> {
    let time_now = chrono::Utc::now().timestamp_millis();
    let url = format!("https://attendance.nbu.edu.cn/api/curriculum/student/getCourse?timeNow={}&pageSize=10", time_now);

    let client = Client::new();
    let response = client.get(&url)
       .header("Cookie", cookie)
       .send()
       .await?;

    let response = response.error_for_status()?;

    let data: Value = response.json().await?;
    if data["code"].as_u64() == Some(30009) {
        println!("{}", data["msg"]);
        return Ok(None);
    }

    let course_infos = data["data"]
       .as_array()
       .unwrap_or(&Vec::new())
       .iter()
       .map(|item| CourseInfo {
            classroom_name: item["classroomName"].as_str().unwrap_or("").to_string(),
            attendance_state: item["attendanceState"].as_i64().unwrap_or(3) as i32,
            course_plan_id: item["coursePlanId"].as_str().unwrap_or("").to_string(),
            attendance_id: item["attendanceId"].as_str().unwrap_or("").to_string(),
        })
       .collect::<Vec<CourseInfo>>();

    Ok(Some(course_infos))
}