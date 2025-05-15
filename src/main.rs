mod  get_class; 

#[tokio::main]
async fn main() {
    use std::io;
    
    println!("请输入cookie:");
    let mut cookie = String::new();
    io::stdin().read_line(&mut cookie).expect("读取输入失败");
    
    match get_class::get_class(cookie.trim()).await {
        Ok(Some(courses)) => {
            println!("成功获取课程信息:");
            for course in courses {
                println!("教室: {}, 考勤状态: {}, 课程ID: {}, 考勤ID: {}", 
                    course.classroom_name, 
                    course.attendance_state, 
                    course.course_plan_id, 
                    course.attendance_id);
            }
        }
        Ok(None) => println!("Cookie失效或未登录"),
        Err(e) => println!("请求失败: {}", e),
    }
}


