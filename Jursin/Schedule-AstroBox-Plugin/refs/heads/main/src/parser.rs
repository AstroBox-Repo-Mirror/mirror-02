//! 课程表配置解析

use std::collections::HashMap;

use serde_json::{json, Map, Value};

/// 将各种来源的时间格式统一为 HH:mm
pub fn normalize_schedule_time(raw: &Value) -> Option<String> {
    match raw {
        Value::Number(n) => {
            let total_seconds = n.as_i64()?;
            if (0..=86399).contains(&total_seconds) {
                let hour = total_seconds / 3600;
                let minute = (total_seconds % 3600) / 60;
                Some(format!("{:02}:{:02}", hour, minute))
            } else {
                None
            }
        }
        other => {
            let text = match other {
                Value::String(s) => s.trim().to_string(),
                _ => other.to_string().trim().to_string(),
            };
            if text.is_empty() {
                return None;
            }
            let parts: Vec<&str> = text.split(':').collect();
            if !(2..=3).contains(&parts.len()) {
                return None;
            }
            let hour: i32 = parts[0].parse().ok()?;
            let minute: i32 = parts[1].parse().ok()?;
            if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
                return None;
            }
            Some(format!("{:02}:{:02}", hour, minute))
        }
    }
}

pub fn time_to_minutes(time: &str) -> i32 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() < 2 {
        return 0;
    }
    let hour: i32 = parts[0].parse().unwrap_or(0);
    let minute: i32 = parts[1].parse().unwrap_or(0);
    hour * 60 + minute
}

// 字段读取辅助，消除重复的 get/and_then/unwrap_or
fn s(obj: &Map<String, Value>, key: &str) -> String {
    obj.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}
fn s_or(obj: &Map<String, Value>, key: &str, default: &str) -> String {
    obj.get(key).and_then(Value::as_str).unwrap_or(default).to_string()
}
fn i(obj: &Map<String, Value>, key: &str) -> i64 {
    obj.get(key).and_then(Value::as_i64).unwrap_or(0)
}
fn i1(obj: &Map<String, Value>, key: &str) -> i64 {
    obj.get(key).and_then(Value::as_i64).unwrap_or(1)
}
fn b(obj: &Map<String, Value>, key: &str) -> bool {
    obj.get(key).and_then(Value::as_bool).unwrap_or(false)
}
fn t(obj: &Map<String, Value>, key: &str) -> Option<String> {
    normalize_schedule_time(obj.get(key).unwrap_or(&Value::Null))
}

/// 按顶层花/方括号深度切分拼接的 JSON 块（正确处理字符串内转义）
fn split_top_level_json_blocks(raw_text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut start: Option<usize> = None;
    let mut depth: i64 = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (i, ch) in raw_text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' | '[' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' | ']' => {
                depth -= 1;
                if depth == 0 && let Some(s) = start {
                    blocks.push(raw_text[s..i + ch.len_utf8()].trim().to_string());
                    start = None;
                }
            }
            _ => {}
        }
    }
    blocks.retain(|b| !b.is_empty());
    blocks
}

fn parse_json(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|e| format!("JSON 解析失败: {}", e))
}

/// WakeUp 课程表 (.wakeup_schedule) -> 标准 JSON
pub fn convert_wakeup_schedule_to_json(wakeup_text: &str) -> Result<Value, String> {
    let mut time_slots_arr: Option<Value> = None;
    let mut table_config: Option<Value> = None;
    let mut course_list_arr: Option<Value> = None;
    let mut course_arr: Option<Value> = None;

    for block in split_top_level_json_blocks(wakeup_text) {
        let trimmed = block.trim();
        if trimmed.starts_with('{') {
            let obj = parse_json(trimmed)?;
            if obj.get("startDate").is_some() || obj.get("maxWeek").is_some() {
                table_config = Some(obj);
            }
        } else if trimmed.starts_with('[') {
            let arr = parse_json(trimmed)?;
            let items = arr
                .as_array()
                .ok_or_else(|| "JSON 结构错误：期望数组".to_string())?;
            if items.is_empty() {
                continue;
            }
            let first = items[0]
                .as_object()
                .ok_or_else(|| "JSON 结构错误：数组元素不是对象".to_string())?;
            if first.contains_key("node")
                && first.contains_key("startTime")
                && first.contains_key("endTime")
            {
                time_slots_arr = Some(arr);
            } else if first.contains_key("id") && first.contains_key("courseName") {
                course_list_arr = Some(arr);
            } else if first.contains_key("id")
                && first.contains_key("day")
                && first.contains_key("startWeek")
                && first.contains_key("endWeek")
            {
                course_arr = Some(arr);
            }
        }
    }

    let (Some(time_slots_arr), Some(table_config), Some(course_list_arr), Some(course_arr)) =
        (time_slots_arr, table_config, course_list_arr, course_arr)
    else {
        return Err("wakeup_schedule 文件结构异常，缺少必需数据块".to_string());
    };

    // 节次
    let mut time_slots = Vec::new();
    for slot in time_slots_arr.as_array().unwrap() {
        let Some(obj) = slot.as_object() else { continue };
        let mut out = Map::new();
        out.insert("number".to_string(), json!(i(obj, "node")));
        out.insert(
            "startTime".to_string(),
            json!(t(obj, "startTime").unwrap_or_else(|| s(obj, "startTime"))),
        );
        out.insert(
            "endTime".to_string(),
            json!(t(obj, "endTime").unwrap_or_else(|| s(obj, "endTime"))),
        );
        time_slots.push(Value::Object(out));
    }

    // 配置
    let table = table_config.as_object().unwrap();
    let mut config = Map::new();
    config.insert("semesterStartDate".to_string(), json!(s(table, "startDate")));
    config.insert("semesterTotalWeeks".to_string(), json!(i(table, "maxWeek")));

    // 课程名表 id -> 名称
    let mut course_id_name_map: HashMap<i64, String> = HashMap::new();
    for c in course_list_arr.as_array().unwrap() {
        let Some(obj) = c.as_object() else { continue };
        course_id_name_map.insert(i(obj, "id"), s(obj, "courseName"));
    }

    // 排课
    let mut courses = Vec::new();
    for c in course_arr.as_array().unwrap() {
        let Some(obj) = c.as_object() else { continue };
        let name = course_id_name_map.get(&i(obj, "id")).cloned().unwrap_or_default();

        let mut out = Map::new();
        out.insert("name".to_string(), json!(name));
        out.insert("teacher".to_string(), json!(s(obj, "teacher")));
        out.insert("position".to_string(), json!(s(obj, "room")));
        out.insert("day".to_string(), json!(i(obj, "day")));

        let start_week = i1(obj, "startWeek");
        let end_week = i1(obj, "endWeek");
        let week_type = i(obj, "type");
        let mut weeks = Vec::new();
        for w in start_week..=end_week {
            if (week_type == 1 && w % 2 == 0) || (week_type == 2 && w % 2 != 0) {
                continue;
            }
            weeks.push(json!(w));
        }
        if weeks.is_empty() {
            return Err(format!("课程 {} 的周数范围无效", name));
        }
        out.insert("weeks".to_string(), Value::Array(weeks));

        let own_time = b(obj, "ownTime");
        out.insert("isCustomTime".to_string(), json!(own_time));
        if own_time {
            let custom_start = t(obj, "startTime")
                .ok_or_else(|| format!("课程 {} 的 startTime 格式不合法", name))?;
            let custom_end = t(obj, "endTime")
                .ok_or_else(|| format!("课程 {} 的 endTime 格式不合法", name))?;
            out.insert("customStartTime".to_string(), json!(custom_start));
            out.insert("customEndTime".to_string(), json!(custom_end));
        } else {
            let start_node = i(obj, "startNode");
            let step = i(obj, "step");
            if start_node <= 0 || step <= 0 {
                return Err(format!("课程 {} 的 startNode/step 不合法", name));
            }
            out.insert("startSection".to_string(), json!(start_node));
            out.insert("endSection".to_string(), json!(start_node + step - 1));
        }
        courses.push(Value::Object(out));
    }

    let mut root = Map::new();
    root.insert("courses".to_string(), Value::Array(courses));
    root.insert("timeSlots".to_string(), Value::Array(time_slots));
    root.insert("config".to_string(), Value::Object(config));
    Ok(Value::Object(root))
}

/// CSES (.yml/.yaml) -> 标准 JSON
pub fn convert_cses_yaml_to_json(yaml_text: &str) -> Result<Value, String> {
    let data: Value = serde_json::to_value(
        serde_yaml::from_str::<serde_yaml::Value>(yaml_text)
            .map_err(|e| format!("YAML 解析失败: {}", e))?,
    )
    .map_err(|e| format!("YAML 转换失败: {}", e))?;
    let data_obj = data
        .as_object()
        .ok_or_else(|| "无效的 YAML 格式".to_string())?;

    // subjects: name -> (teacher, room)
    let mut subject_map: HashMap<String, (String, String)> = HashMap::new();
    if let Some(subjects) = data_obj.get("subjects").and_then(Value::as_array) {
        for sub in subjects {
            if let Some(o) = sub.as_object() {
                subject_map.insert(s(o, "name"), (s(o, "teacher"), s(o, "room")));
            }
        }
    }

    let schedules = data_obj
        .get("schedules")
        .and_then(Value::as_array)
        .ok_or_else(|| "缺少必填项 schedules".to_string())?;

    // 全局时间轴
    let mut all_unique_times: Vec<String> = Vec::new();
    for schedule in schedules {
        if let Some(o) = schedule.as_object()
            && let Some(classes) = o.get("classes").and_then(Value::as_array)
        {
            for cls in classes {
                if let Some(co) = cls.as_object() {
                    let subject = s(co, "subject");
                    let start = t(co, "start_time")
                        .ok_or_else(|| format!("课程 {} 的 start_time 格式不合法", subject))?;
                    let end = t(co, "end_time")
                        .ok_or_else(|| format!("课程 {} 的 end_time 格式不合法", subject))?;
                    if !all_unique_times.contains(&start) {
                        all_unique_times.push(start);
                    }
                    if !all_unique_times.contains(&end) {
                        all_unique_times.push(end);
                    }
                }
            }
        }
    }
    all_unique_times.sort_by_key(|t| time_to_minutes(t));
    if all_unique_times.len() < 2 {
        return Err("时间轴数据不足，无法生成 timeSlots".to_string());
    }

    let time_number_map: HashMap<String, usize> = all_unique_times
        .iter()
        .enumerate()
        .map(|(i, t)| (t.clone(), i + 1))
        .collect();

    let mut time_slots = Vec::new();
    for i in 0..all_unique_times.len() - 1 {
        let mut obj = Map::new();
        obj.insert("number".to_string(), json!(i + 1));
        obj.insert("startTime".to_string(), json!(all_unique_times[i]));
        obj.insert("endTime".to_string(), json!(all_unique_times[i + 1]));
        time_slots.push(Value::Object(obj));
    }

    let mut courses = Vec::new();
    for schedule in schedules {
        if let Some(o) = schedule.as_object() {
            let day = i1(o, "enable_day");
            let weeks_type = s_or(o, "weeks", "all");
            if let Some(classes) = o.get("classes").and_then(Value::as_array) {
                for cls in classes {
                    if let Some(co) = cls.as_object() {
                        let subject = s(co, "subject");
                        let start = t(co, "start_time")
                            .ok_or_else(|| format!("课程 {} 的 start_time 格式不合法", subject))?;
                        let end = t(co, "end_time")
                            .ok_or_else(|| format!("课程 {} 的 end_time 格式不合法", subject))?;
                        let start_number = *time_number_map
                            .get(&start)
                            .ok_or_else(|| format!("课程 {} 时间 [{} - {}] 在时间轴中未找到", subject, start, end))?;
                        let end_number = *time_number_map
                            .get(&end)
                            .ok_or_else(|| format!("课程 {} 时间 [{} - {}] 在时间轴中未找到", subject, start, end))?;
                        if end_number <= start_number {
                            return Err(format!("课程 {} 时间 [{} - {}] 起止顺序不合法", subject, start, end));
                        }
                        let (teacher, room) = subject_map.get(&subject).cloned().unwrap_or_default();
                        let mut course = Map::new();
                        course.insert("name".to_string(), json!(subject));
                        course.insert("teacher".to_string(), json!(teacher));
                        course.insert("position".to_string(), json!(room));
                        course.insert("day".to_string(), json!(day));
                        course.insert("weekType".to_string(), json!(weeks_type));
                        course.insert("isCustomTime".to_string(), json!(false));
                        course.insert("startSection".to_string(), json!(start_number as i64));
                        course.insert("endSection".to_string(), json!(end_number as i64 - 1));
                        courses.push(Value::Object(course));
                    }
                }
            }
        }
    }

    let mut config = Map::new();
    config.insert("semesterStartDate".to_string(), json!(""));
    config.insert("semesterTotalWeeks".to_string(), json!(""));

    let mut root = Map::new();
    root.insert("courses".to_string(), Value::Array(courses));
    root.insert("timeSlots".to_string(), Value::Array(time_slots));
    root.insert("config".to_string(), Value::Object(config));
    Ok(Value::Object(root))
}

/// 校验标准配置，返回错误信息（None 表示通过）
pub fn validate_schedule_config(root: &Value) -> Option<String> {
    let root_obj = match root.as_object() {
        Some(o) => o,
        None => return Some("配置文件不是有效对象".to_string()),
    };
    let courses = match root_obj.get("courses") {
        Some(Value::Array(a)) => a,
        _ => return Some("缺少必填项 courses".to_string()),
    };
    let time_slots = match root_obj.get("timeSlots") {
        Some(Value::Array(a)) => a,
        _ => return Some("缺少必填项 timeSlots".to_string()),
    };
    if let Some(err) = validate_courses(courses) {
        return Some(err);
    }
    if let Some(err) = validate_time_slots(time_slots) {
        return Some(err);
    }
    None
}

fn validate_courses(courses: &[Value]) -> Option<String> {
    for (i, course) in courses.iter().enumerate() {
        let obj = match course.as_object() {
            Some(o) => o,
            None => return Some(format!("courses[{}] 必须是对象", i)),
        };
        let name = s(obj, "name");
        if name.is_empty() {
            return Some(format!("courses[{}].name 必填", i));
        }
        if !obj.contains_key("day") {
            return Some(format!("courses[{}].day 必填", i));
        }
        if !obj.contains_key("weeks") && !obj.contains_key("weekType") {
            return Some(format!("courses[{}].weeks 或 weekType 必填", i));
        }
        if obj.contains_key("weeks") {
            let weeks = obj.get("weeks");
            let empty = match weeks {
                Some(Value::Array(a)) => a.is_empty(),
                _ => true,
            };
            if empty {
                return Some(format!("courses[{}].weeks 不能为空", i));
            }
        }
        if obj.contains_key("weekType") {
            let week_type = s(obj, "weekType");
            if week_type.is_empty() {
                return Some(format!("courses[{}].weekType 不能为空", i));
            }
        }
        let is_custom_time = b(obj, "isCustomTime");
        if is_custom_time {
            let custom_start = s(obj, "customStartTime");
            if custom_start.is_empty() {
                return Some(format!("courses[{}].customStartTime 必填", i));
            }
            let custom_end = s(obj, "customEndTime");
            if custom_end.is_empty() {
                return Some(format!("courses[{}].customEndTime 必填", i));
            }
        } else {
            if !obj.contains_key("startSection") {
                return Some(format!("courses[{}].startSection 必填", i));
            }
            if !obj.contains_key("endSection") {
                return Some(format!("courses[{}].endSection 必填", i));
            }
        }
    }
    None
}

fn validate_time_slots(time_slots: &[Value]) -> Option<String> {
    for (i, slot) in time_slots.iter().enumerate() {
        let obj = match slot.as_object() {
            Some(o) => o,
            None => return Some(format!("timeSlots[{}] 必须是对象", i)),
        };
        if !obj.contains_key("number") {
            return Some(format!("timeSlots[{}].number 必填", i));
        }
        let start = s(obj, "startTime");
        if start.is_empty() {
            return Some(format!("timeSlots[{}].startTime 必填", i));
        }
        let end = s(obj, "endTime");
        if end.is_empty() {
            return Some(format!("timeSlots[{}].endTime 必填", i));
        }
    }
    None
}

/// 按白名单字段重排输出，剔除多余字段（控制 payload 体积）
pub fn sanitize_schedule_payload(root: &Value) -> Value {
    let root_obj = root.as_object().cloned().unwrap_or_default();
    let mut sanitized_root = Map::new();

    // 课程
    let mut sanitized_courses = Vec::new();
    for course in root_obj
        .get("courses")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        if let Some(obj) = course.as_object() {
            let mut out = Map::new();
            out.insert("name".to_string(), json!(s(obj, "name")));
            out.insert("day".to_string(), json!(i(obj, "day")));
            out.insert("isCustomTime".to_string(), json!(b(obj, "isCustomTime")));
            if obj.contains_key("teacher") {
                out.insert("teacher".to_string(), json!(s(obj, "teacher")));
            }
            if obj.contains_key("position") {
                out.insert("position".to_string(), json!(s(obj, "position")));
            }
            if obj.contains_key("weeks") {
                out.insert(
                    "weeks".to_string(),
                    obj.get("weeks").cloned().unwrap_or(Value::Array(vec![])),
                );
            }
            if obj.contains_key("weekType") {
                out.insert("weekType".to_string(), json!(s(obj, "weekType")));
            }
            if b(obj, "isCustomTime") {
                out.insert("customStartTime".to_string(), json!(s(obj, "customStartTime")));
                out.insert("customEndTime".to_string(), json!(s(obj, "customEndTime")));
            } else {
                out.insert("startSection".to_string(), json!(i(obj, "startSection")));
                out.insert("endSection".to_string(), json!(i(obj, "endSection")));
            }
            sanitized_courses.push(Value::Object(out));
        }
    }
    sanitized_root.insert("courses".to_string(), Value::Array(sanitized_courses));

    // 节次
    let mut sanitized_time_slots = Vec::new();
    for slot in root_obj
        .get("timeSlots")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        if let Some(obj) = slot.as_object() {
            let mut out = Map::new();
            out.insert("number".to_string(), json!(i(obj, "number")));
            out.insert("startTime".to_string(), json!(s(obj, "startTime")));
            out.insert("endTime".to_string(), json!(s(obj, "endTime")));
            sanitized_time_slots.push(Value::Object(out));
        }
    }
    sanitized_root.insert("timeSlots".to_string(), Value::Array(sanitized_time_slots));

    // 配置（仅保留存在字段）
    if let Some(config) = root_obj.get("config").and_then(Value::as_object) {
        let mut out = Map::new();
        for key in ["semesterStartDate", "semesterTotalWeeks"] {
            if let Some(v) = config.get(key) {
                out.insert(key.to_string(), v.clone());
            }
        }
        sanitized_root.insert("config".to_string(), Value::Object(out));
    }

    Value::Object(sanitized_root)
}

/// 判断是否为星链课表导出JSON
pub fn is_starlink_json(text: &str) -> bool {
    let value = match serde_json::from_str::<Value>(text) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let root = match value.as_object() {
        Some(o) => o,
        None => return false,
    };
    // 有timeSlots则为标准格式，不是星链导出
    if root.contains_key("timeSlots") {
        return false;
    }
    // 检查courses数组中的元素是否有weekday字段
    match root.get("courses").and_then(Value::as_array) {
        Some(arr) => arr.first().and_then(Value::as_object).is_some_and(|c| c.contains_key("weekday")),
        None => false,
    }
}

/// 判断是否为星链时间表导出JSON
pub fn is_timetable_json(text: &str) -> bool {
    let value = match serde_json::from_str::<Value>(text) {
        Ok(v) => v,
        Err(_) => return false,
    };
    value.get("type").and_then(Value::as_str) == Some("timetable")
        && value.get("data").and_then(|d| d.get("items")).and_then(Value::as_array).is_some()
}

/// 将星链时间表转换为标准timeSlots
pub fn convert_timetable_to_time_slots(text: &str) -> Result<Vec<Value>, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("JSON解析失败: {}", e))?;

    let items = value
        .get("data")
        .and_then(|d| d.get("items"))
        .and_then(Value::as_array)
        .ok_or_else(|| "时间表缺少data.items".to_string())?;

    let mut time_slots = Vec::new();
    for item in items {
        let obj = match item.as_object() {
            Some(o) => o,
            None => continue,
        };
        let section = obj.get("section").and_then(Value::as_i64).unwrap_or(0);
        let start_h = obj.get("startHour").and_then(Value::as_i64).unwrap_or(0);
        let start_m = obj.get("startMinute").and_then(Value::as_i64).unwrap_or(0);
        let end_h = obj.get("endHour").and_then(Value::as_i64).unwrap_or(0);
        let end_m = obj.get("endMinute").and_then(Value::as_i64).unwrap_or(0);

        time_slots.push(json!({
            "number": section,
            "startTime": format!("{:02}:{:02}", start_h, start_m),
            "endTime": format!("{:02}:{:02}", end_h, end_m)
        }));
    }

    time_slots.sort_by_key(|s| s.get("number").and_then(Value::as_i64).unwrap_or(0));
    Ok(time_slots)
}

/// 星链课表JSON转换为标准格式
pub fn convert_starlink_json_to_json(text: &str) -> Result<Value, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("JSON解析失败: {}", e))?;

    // 1. 转换课程数据
    let courses = convert_starlink_courses(&value)?;

    // 2. 转换时间段数据（可能为空）
    let time_slots = convert_starlink_time_slots(&value);

    // 3. 提取配置信息（可能为空）
    let config = extract_optional_config(&value);

    let mut root = json!({
        "courses": courses,
        "timeSlots": time_slots,
    });
    if let Some(cfg) = config {
        root["config"] = cfg;
    }

    Ok(root)
}

/// 转换星链课程数据
fn convert_starlink_courses(data: &Value) -> Result<Vec<Value>, String> {
    let courses_array = data
        .get("courses")
        .and_then(Value::as_array)
        .ok_or_else(|| "缺少courses数组".to_string())?;

    let mut courses = Vec::new();

    for course in courses_array {
        let course_obj = match course.as_object() {
            Some(o) => o,
            None => continue,
        };

        let name = s(course_obj, "name");
        if name.is_empty() {
            continue;
        }

        let day = i(course_obj, "weekday");
        let start_section = i(course_obj, "startSection");
        let end_section = i(course_obj, "endSection");

        let weeks = course_obj
            .get("weeks")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|w| w.as_i64())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        courses.push(json!({
            "name": name,
            "teacher": s(course_obj, "teacher"),
            "position": s(course_obj, "location").trim_start_matches('@').to_string(),
            "day": day,
            "startSection": start_section,
            "endSection": end_section,
            "weeks": weeks
        }));
    }

    Ok(courses)
}

/// 从sectionMinutes转换时间段数据，无则返回空Vec
fn convert_starlink_time_slots(data: &Value) -> Vec<Value> {
    let section_minutes = match data.get("sectionMinutes").and_then(Value::as_object) {
        Some(m) => m,
        None => return Vec::new(),
    };

    let mut time_slots = Vec::new();

    for (number_str, time_range) in section_minutes {
        let number: i64 = match number_str.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        if let Some(range) = time_range.as_array() {
            if range.len() >= 2 {
                let start_minutes = range[0].as_i64().unwrap_or(0) as i32;
                let end_minutes = range[1].as_i64().unwrap_or(0) as i32;

                time_slots.push(json!({
                    "number": number,
                    "startTime": minutes_to_time_str(start_minutes),
                    "endTime": minutes_to_time_str(end_minutes)
                }));
            }
        }
    }

    time_slots.sort_by_key(|slot| slot.get("number").and_then(Value::as_i64).unwrap_or(0));
    time_slots
}

/// 将分钟数转换为HH:mm格式
fn minutes_to_time_str(minutes: i32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    format!("{:02}:{:02}", hours, mins)
}

/// 提取星链配置信息，无有效数据则返回 None
fn extract_optional_config(data: &Value) -> Option<Value> {
    let start_date = data
        .get("startDate")
        .and_then(Value::as_str)
        .map(|s| {
            if s.len() >= 10 {
                s[..10].to_string()
            } else {
                s.to_string()
            }
        })
        .filter(|s| !s.is_empty())?;

    let total_weeks = data
        .get("totalWeeks")
        .and_then(Value::as_i64)
        .filter(|&w| w > 0)?;

    Some(json!({
        "semesterStartDate": start_date,
        "semesterTotalWeeks": total_weeks
    }))
}
