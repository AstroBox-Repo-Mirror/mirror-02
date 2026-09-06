//! 引擎:定时器布防、事件分发、设备发现与互联注册、同步与推送。
//!
//! 定时器 payload:
//! - `sync` 每 60s:查余额 → 导入用量 → 推快照(固定 60s,含变化检测)
//! - `housekeeping` 30s:刷新已连接设备、注册互联接收、刷新时区

use astrobox_ng_wit::astrobox::psys_host::{device, interconnect, os, register, thirdpartyapp, timer};
use astrobox_ng_wit::exports::astrobox::psys_plugin::event::EventType;

use serde_json::Value;

use crate::state::{self, DEFAULT_BASE_URL, DEFAULT_PKG, DEFAULT_PLATFORM_BASE};
use crate::{dates, deepseek, import, snapshot};

/// 固定推送周期(秒)
const SYNC_INTERVAL_SECS: u64 = 60;
const HOUSEKEEPING_INTERVAL_SECS: u64 = 30;

/// 快应用互联消息短窗口去重:vela 的 onopen/getReadyState 或宿主事件重复派发
/// 可能让同一动作产生多条消息,窗口内的重复消息只处理第一条。
const INTERCONNECT_DEDUP_SECS: i64 = 3;

/// 首次同步延迟(ms):不等第一个 60s 周期,加载后尽快出数据
const INITIAL_SYNC_DELAY_MS: u64 = 5_000;

/// 用量导出窗口(天):覆盖今天即可,多取几天留时区余量
const EXPORT_WINDOW_DAYS: i64 = 7;

/// on_load 中调用(block_on):加载持久化、注册互联接收、布防定时器。
pub async fn init() {
    state::init_from_disk();

    let off = os::timezone_offset_minutes().await;
    state::lock().tz_offset_min = off;
    tracing::info!("[init] 宿主时区偏移 {off} 分钟");

    refresh_device().await;
    arm_timers().await;

    let status = {
        let a = state::lock();
        match (&a.device_addr, a.recv_registered) {
            (None, _) => "插件已启动:未连接设备".to_string(),
            (Some(_), false) => {
                "插件已启动:互联接收未注册成功(检查权限,housekeeping 会重试)".to_string()
            }
            (Some(_), true) => "插件已启动:等待首次同步(约 5 秒)".to_string(),
        }
    };
    state::set_status(&status);
}

/// 布防定时器(周期固定,无参数)。
pub async fn arm_timers() {
    // 清理旧定时器
    let old: Vec<u64> = state::lock().timer_ids.values().copied().collect();
    for id in old {
        timer::clear_timer(id).await;
    }

    let mut ids = std::collections::BTreeMap::new();
    ids.insert(
        "sync".to_string(),
        timer::set_interval(SYNC_INTERVAL_SECS * 1000, "sync").await,
    );
    ids.insert(
        "housekeeping".to_string(),
        timer::set_interval(HOUSEKEEPING_INTERVAL_SECS * 1000, "housekeeping").await,
    );
    // 首次同步尽快执行(单次 timeout,自动移除)
    timer::set_timeout(INITIAL_SYNC_DELAY_MS, "sync").await;

    state::lock().timer_ids = ids;
    tracing::info!(
        "[timer] sync {SYNC_INTERVAL_SECS}s · housekeeping {HOUSEKEEPING_INTERVAL_SECS}s"
    );
}

/// on_event 分发。
pub async fn handle_event(event_type: EventType, payload: &str) {
    match event_type {
        EventType::Timer => {
            // {"timerId":..,"kind":"interval","payload":"sync"}
            let which = serde_json::from_str::<Value>(payload)
                .ok()
                .and_then(|v| v.get("payload").and_then(Value::as_str).map(String::from));
            match which.as_deref() {
                Some("sync") => sync_now().await,
                Some("housekeeping") => housekeeping().await,
                other => tracing::debug!("[timer] 未识别的定时器载荷: {other:?}"),
            }
        }
        EventType::InterconnectMessage => {
            // 手环侧快应用打开/请求刷新 → 立即回一版快照
            handle_interconnect_message(payload).await;
        }
        other => tracing::debug!("[event] {:?} len={}", other, payload.len()),
    }
}

/// 快应用互联消息应答:手环打开快应用/请求刷新时立即强推一版快照
pub async fn handle_interconnect_message(payload: &str) {
    let now = dates::unix_now();
    let duplicate = {
        let a = state::lock();
        a.last_interconnect_at
            .map(|t| now.saturating_sub(t) < INTERCONNECT_DEDUP_SECS)
            .unwrap_or(false)
    };
    if duplicate {
        tracing::info!(
            "[interconnect] 忽略 {INTERCONNECT_DEDUP_SECS}s 内的重复快应用消息(len={})",
            payload.len()
        );
        return;
    }
    state::lock().last_interconnect_at = Some(now);
    tracing::info!(
        "[interconnect] 收到快应用消息(len={}) 立即强推快照",
        payload.len()
    );
    push_now(true).await;
}

/// 刷新已连接设备;设备变化时重新注册互联接收。
pub async fn refresh_device() {
    let devices = device::get_connected_device_list().await;
    let addr = devices.first().map(|d| d.addr.clone());

    let need_register = {
        let mut a = state::lock();
        if addr != a.device_addr {
            a.recv_registered = false;
            a.device_addr = addr.clone();
        }
        !a.recv_registered && addr.is_some()
    };

    match &addr {
        Some(a) => tracing::info!("[device] 已连接设备: {}({a})", devices[0].name),
        None => tracing::debug!("[device] 无已连接设备"),
    }

    if need_register && let Some(a) = &addr {
        match register::register_interconnect_recv(a, DEFAULT_PKG).await {
            Ok(()) => {
                state::lock().recv_registered = true;
                tracing::info!(
                    "[interconnect] 已注册互联接收: {a} {DEFAULT_PKG},等待快应用上行消息"
                );
            }
            Err(()) => {
                tracing::warn!(
                    "[interconnect] 注册互联接收失败(检查 register_interconnect_recv 权限);手环快应用的上行消息将无法到达插件"
                );
            }
        }
    }
}

async fn housekeeping() {
    refresh_device().await;
    let off = os::timezone_offset_minutes().await;
    state::lock().tz_offset_min = off;
}

/// 一次完整同步:查余额 → 导入用量 → 推送快照。
///
/// 任一步失败都不阻断后续步骤(余额失败仍推用量,反之亦然)。
pub async fn sync_now() {
    tracing::info!("[sync] 开始同步");
    balance_now().await;
    export_now().await;
    push_now(false).await;
}

/// 查余额(需要 API Key)。
pub async fn balance_now() {
    let key = state::lock().settings.api_key.clone();
    if key.is_empty() {
        tracing::debug!("[sync] 未设置 API Key,跳过余额查询");
        return;
    }
    let now = dates::unix_now();
    match deepseek::fetch_balance(DEFAULT_BASE_URL, &key, now) {
        Ok(Some(info)) => {
            let total = info.total;
            state::lock().data.balance = Some(info);
            state::save_data();
            state::set_status(&format!("余额已更新: ¥{total:.2}"));
        }
        Ok(None) => state::set_status("余额接口不可用(检查 API Key)"),
        Err(e) => state::set_status(&format!("余额查询失败: {e:#}")),
    }
}

/// 导出并导入用量(需要平台 Token)。
pub async fn export_now() {
    let token = state::lock().settings.platform_token.clone();
    if token.is_empty() {
        tracing::debug!("[sync] 未设置平台 token,跳过用量导入");
        return;
    }
    let tz = state::lock().tz_offset_min;
    let (start, end) = deepseek::default_window(EXPORT_WINDOW_DAYS, tz);
    match deepseek::fetch_export_zip(DEFAULT_PLATFORM_BASE, &token, start, end) {
        Ok(bytes) => match import::import_zip_bytes(&bytes, tz) {
            Ok((days, models, replaced)) => {
                let now = dates::unix_now();
                state::lock().data.last_import_at = Some(now);
                state::save_data();
                state::set_status(&format!(
                    "用量导入成功: {days} 天 · {models} 模型 · 替换 {replaced} 行"
                ));
            }
            Err(e) => state::set_status(&format!("用量导入失败: {e}")),
        },
        Err(e) => state::set_status(&format!("用量导出失败: {e:#}")),
    }
}

/// 构建并推送快照到手环快应用。
///
/// `force`:
/// - `false`(定时推送):快照业务数据与上次成功推送相同时跳过;
/// - `true`(手环打开应用 / 设置页手动推送):忽略变化检测,立即回复。
///
/// 推送前通过 `thirdpartyapp` 查询目标快应用是否已安装;确认未安装时
/// 直接跳过并给出明确状态,不再发送必然失败的消息。
pub async fn push_now(force: bool) {
    let (addr, pkg, json, signature) = {
        let a = state::lock();
        let snap = snapshot::build_snapshot(&a.data, "deepseek", a.tz_offset_min);
        let signature = snapshot::stable_signature(&snap);
        let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".into());
        (a.device_addr.clone(), DEFAULT_PKG.to_string(), json, signature)
    };

    let Some(addr) = addr else {
        state::set_status("未连接设备,跳过推送(请先在 AstroBox 连接手环)");
        return;
    };

    // 定时推送做变化检测:同一设备 + 同一业务数据 → 跳过。
    // 手环打开应用/手动按钮走 force=true,始终回复。
    if !force {
        let unchanged = {
            let a = state::lock();
            a.last_pushed_device.as_deref() == Some(addr.as_str())
                && a.last_pushed_signature.as_deref() == Some(signature.as_str())
        };
        if unchanged {
            tracing::info!("[push] 快照无变化({} 字节),跳过推送", json.len());
            state::set_status(&format!("快照无变化,跳过推送({} 字节)", json.len()));
            return;
        }
    }

    // 推送前检测:确认手环端已安装目标快应用。
    // 查询失败(权限/设备无响应)时降级为继续推送,不阻断原有链路。
    match thirdpartyapp::get_thirdparty_app_list(&addr).await {
        Ok(apps) => match apps.iter().find(|app| app.package_name == pkg) {
            Some(app) => tracing::info!(
                "[precheck] 目标应用已安装: {} version_code={} app_name={}",
                app.package_name,
                app.version_code,
                app.app_name
            ),
            None => {
                let installed: Vec<&str> = apps.iter().map(|a| a.package_name.as_str()).collect();
                tracing::warn!("[precheck] 设备 {addr} 未安装 {pkg};已安装快应用: {installed:?}");
                state::set_status(&format!(
                    "推送失败:手环未安装 {pkg},请先通过 AstroBox 安装 vela 快应用"
                ));
                return;
            }
        },
        Err(()) => {
            tracing::warn!(
                "[precheck] 无法获取第三方应用列表(检查 thirdpartyapp 权限/设备响应),跳过检测继续推送"
            );
        }
    }

    match interconnect::send_qaic_message(&addr, &pkg, &json).await {
        Ok(()) => {
            let t = dates::unix_now();
            {
                let mut a = state::lock();
                a.last_push_at = Some(t);
                a.last_pushed_signature = Some(signature);
                a.last_pushed_device = Some(addr.clone());
            }
            tracing::info!("[push] 已发送快照 {} 字节 → {addr} {pkg}", json.len());
            state::set_status(&format!("已推送快照 {} 字节 → {pkg}", json.len()));
        }
        Err(()) => {
            tracing::warn!("[push] send_qaic_message 失败: {addr} {pkg}");
            state::set_status("推送失败:设备不在线/未安装快应用/未授权 interconnect");
        }
    }
}
