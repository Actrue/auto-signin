mod get_class;
mod attendance;

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// 直接粘贴cookie值
    #[arg(short = 'c', long, required = true)]
    cookie: String,

    /// 启用监听模式
    #[arg(short = 'w', long)]
    watch: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cookie = cli.cookie.trim().to_string();
    
    let check_attendance = || async {
    match get_class::get_class(&cookie).await {
        Ok(Some(courses)) => {
            for course in courses {
                if course.attendance_state == 0 {
                    let params = attendance::SignParams {
                        course_plan_id: course.course_plan_id,
                        attendance_id: course.attendance_id,
                        cookie: cookie.to_string(),
                    };
                    return Some(params);
                }
            }
            None
        }
        _ => None,
    }
};

if cli.watch {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Some(params) = check_attendance().await {
            match attendance::sign_attendance(&params).await {
                Ok(Some(code)) => println!("签到成功，使用的课程码: {}", code),
                Ok(None) => println!("未找到有效课程码"),
                Err(e) => println!("签到失败: {}", e),
            }
            break;
        }
    }
} else {
    match get_class::get_class(&cookie).await {
        Ok(Some(courses)) => {
            println!("成功获取课程信息:");
            for course in courses {
    println!("教室: {}, 考勤状态: {}", 
        course.classroom_name, 
        match course.attendance_state {
            3 => "未开课",
            0 => "未签到",
            1 => "已签到",
            2 => "迟到",
            _ => "未知状态"
        });

    if course.attendance_state == 0 {
        let params = attendance::SignParams {
            course_plan_id: course.course_plan_id,
            attendance_id: course.attendance_id,
            cookie: cookie.to_string(),
        };

        match attendance::sign_attendance(&params).await {
            Ok(Some(code)) => println!("签到成功，使用的课程码: {}", code),
            Ok(None) => println!("未找到有效课程码"),
            Err(e) => println!("签到失败: {}", e),
        }
    }
}
        }
        Ok(None) => println!("Cookie失效或未登录"),
        Err(e) => println!("请求失败: {}", e),
    }
}
}


