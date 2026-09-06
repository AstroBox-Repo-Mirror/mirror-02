//! 插件 UI(设置页):仅两个凭据输入 + 保存/立即同步 + 状态行。

use astrobox_ng_wit::astrobox::psys_host::ui::{self, ElementType, Event, FlexDirection};

use serde_json::Value;

use crate::{engine, state};

pub const INPUT_API_KEY: &str = "input-api-key";
pub const INPUT_PLATFORM_TOKEN: &str = "input-platform-token";
pub const BTN_SAVE: &str = "btn-save";
pub const BTN_SYNC_NOW: &str = "btn-sync-now";

pub fn render_page(element_id: &str) {
    state::lock().page_element_id = Some(element_id.to_string());
    let el = build_page();
    ui::render(element_id, el);
}

fn build_page() -> ui::Element {
    let (api_key, token, status) = {
        let a = state::lock();
        (
            a.settings.api_key.clone(),
            a.settings.platform_token.clone(),
            a.status.clone(),
        )
    };

    let title = ui::Element::new(ElementType::P, Some("dswatch-sync 设置")).size(24);

    let key_label =
        ui::Element::new(ElementType::P, Some("DeepSeek API Key（用于余额查询）"))
            .size(14)
            .margin_bottom(6);
    let key_input = ui::Element::new(ElementType::Input, Some(api_key.as_str()))
        .width_full()
        .padding(8)
        .radius(6)
        .border(1, "#3a4050")
        .margin_bottom(14)
        .on(Event::Input, INPUT_API_KEY);

    let token_label =
        ui::Element::new(ElementType::P, Some("Bearer Token（用于获取用量信息）"))
            .size(14)
            .margin_bottom(6);
    let token_input = ui::Element::new(ElementType::Input, Some(token.as_str()))
        .width_full()
        .padding(8)
        .radius(6)
        .border(1, "#3a4050")
        .margin_bottom(14)
        .on(Event::Input, INPUT_PLATFORM_TOKEN);

    // 引导文案:分步说明 Bearer Token 的获取方式
    let guide_title = ui::Element::new(ElementType::P, Some("获取方式"))
        .size(13)
        .margin_bottom(4);
    let guide_lines = [
        "① 浏览器打开 DeepSeek 后台-用量信息",
        "② 点击 F12 打开开发者工具，切换到 Network 标签",
        "③ 点击页面上的导出按钮",
        "④ export 请求中找到 authorization 字段，复制 Bearer 开头的信息",
    ];
    let mut guide = ui::Element::new(ElementType::Div, None)
        .flex()
        .flex_direction(FlexDirection::Column)
        .width_full()
        .margin_bottom(14)
        .child(guide_title);
    for (i, line) in guide_lines.iter().enumerate() {
        let is_last = i + 1 == guide_lines.len();
        let line_el = ui::Element::new(ElementType::P, Some(line))
            .size(11)
            .text_color("#9aa3b2")
            .margin_bottom(if is_last { 0 } else { 3 });
        guide = guide.child(line_el);
    }

    let save_btn = ui::Element::new(ElementType::Button, Some("保存设置"))
        .bg("#2B5BE8")
        .text_color("#FFFFFF")
        .padding(10)
        .radius(6)
        .margin_bottom(8)
        .on(Event::Click, BTN_SAVE);

    let sync_btn = ui::Element::new(ElementType::Button, Some("立即同步并推送"))
        .bg("#28A745")
        .text_color("#FFFFFF")
        .padding(10)
        .radius(6)
        .on(Event::Click, BTN_SYNC_NOW);

    let status_text = ui::Element::new(ElementType::P, Some(status.as_str()))
        .size(12)
        .text_color("#9aa3b2")
        .margin_top(10);

    ui::Element::new(ElementType::Div, None)
        .flex()
        .flex_direction(FlexDirection::Column)
        .width_full()
        .padding(16)
        .child(title)
        .child(key_label)
        .child(key_input)
        .child(token_label)
        .child(token_input)
        .child(guide)
        .child(save_btn)
        .child(sync_btn)
        .child(status_text)
}

pub async fn handle_ui_event(event_id: &str, evtype: &ui::Event, payload: &str) {
    match evtype {
        ui::Event::Input => match event_id {
            INPUT_API_KEY => {
                let v = input_value(payload);
                state::lock().settings.api_key = v;
            }
            INPUT_PLATFORM_TOKEN => {
                let v = input_value(payload);
                state::lock().settings.platform_token = v;
            }
            _ => {}
        },
        ui::Event::Click => match event_id {
            BTN_SAVE => {
                state::save_settings();
                state::set_status("设置已保存");
            }
            BTN_SYNC_NOW => {
                engine::sync_now().await;
            }
            _ => {}
        },
        _ => {}
    }
    // 重绘页面。Input 事件除外:输入态下整页重绘会把输入框 value 重置为
    // state 里的旧值(输入内容会被清空/抖动),部分 AstroBox 版本会因此
    // 触发 UI 引擎异常;输入值已写入 state,下次 Click 事件时自然生效。
    if !matches!(evtype, ui::Event::Input) {
        if let Some(id) = state::lock().page_element_id.clone() {
            render_page(&id);
        }
    }
}

/// 从输入事件载荷中提取用户输入值。
/// 载荷形如 `{"type":"input","value":"sk-..."}`。
fn input_value(payload: &str) -> String {
    serde_json::from_str::<Value>(payload)
        .ok()
        .and_then(|v| match v.get("value") {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Number(n)) => Some(n.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}
