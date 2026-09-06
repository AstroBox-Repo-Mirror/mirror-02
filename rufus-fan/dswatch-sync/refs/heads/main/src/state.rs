//! 全局状态与持久化

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use serde::{Deserialize, Serialize};

use crate::snapshot::BalanceInfo;

pub const SETTINGS_FILE: &str = "settings.json";
pub const DATA_FILE: &str = "data.json";

/// 数据保留天数
pub const RETAIN_DAYS: i64 = 90;

/// 目标快应用包名(DS Watch)
pub const DEFAULT_PKG: &str = "com.dswatch.periodreminder";

pub const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";
pub const DEFAULT_PLATFORM_BASE: &str = "https://platform.deepseek.com";

/// 设置项:只保留必要凭据,其余全部硬编码(推送周期 60s、包名、接口地址)。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// DeepSeek API Key(余额接口)
    pub api_key: String,
    /// DeepSeek 开放平台 token(用量导出接口,浏览器 F12 复制)
    pub platform_token: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            platform_token: String::new(),
        }
    }
}

/// 单日单模型用量行(不含日期,日期为 days map 的键)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayModelUsage {
    pub model: String,
    pub calls: u64,
    pub hit_tokens: u64,
    pub miss_tokens: u64,
    pub output_tokens: u64,
    /// Paid + Granted 合计(CNY)
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DataFile {
    pub version: u32,
    pub balance: Option<BalanceInfo>,
    pub last_import_at: Option<i64>,
    /// date(YYYY-MM-DD) → 行列表(每行一个模型)
    pub days: BTreeMap<String, Vec<DayModelUsage>>,
}

impl Default for DataFile {
    fn default() -> Self {
        Self {
            version: 1,
            balance: None,
            last_import_at: None,
            days: BTreeMap::new(),
        }
    }
}

pub struct App {
    pub settings: Settings,
    pub data: DataFile,
    /// 宿主时区相对 UTC 分钟数(来自 psys_host::os)
    pub tz_offset_min: i32,
    /// 当前已连接设备地址(取已连接列表第一个)
    pub device_addr: Option<String>,
    /// 是否已对该设备注册 interconnect 接收
    pub recv_registered: bool,
    /// 插件页面渲染 id(on_ui_render 传入,重绘用)
    pub page_element_id: Option<String>,
    /// payload → timer id
    pub timer_ids: BTreeMap<String, u64>,
    /// UI 状态行(最近一次动作结果)
    pub status: String,
    pub status_at: i64,
    pub last_push_at: Option<i64>,
    /// 最近一次成功推送的快照稳定签名(变化检测用,不含 generatedAt/freshness)
    pub last_pushed_signature: Option<String>,
    /// 最近一次成功推送的目标设备(换设备后即使数据相同也要重推)
    pub last_pushed_device: Option<String>,
    /// 最近一次收到快应用互联消息的 Unix 秒(短窗口去重,避免重复强推)
    pub last_interconnect_at: Option<i64>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            data: DataFile::default(),
            tz_offset_min: 480, // 缺省 +08:00,on_load 后用宿主值覆盖
            device_addr: None,
            recv_registered: false,
            page_element_id: None,
            timer_ids: BTreeMap::new(),
            status: "未初始化".into(),
            status_at: 0,
            last_push_at: None,
            last_pushed_signature: None,
            last_pushed_device: None,
            last_interconnect_at: None,
        }
    }
}

static APP: OnceLock<Mutex<App>> = OnceLock::new();

pub fn app() -> &'static Mutex<App> {
    APP.get_or_init(|| Mutex::new(App::default()))
}

pub fn lock() -> MutexGuard<'static, App> {
    app()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 从插件目录加载 settings.json / data.json;文件缺失或损坏时使用默认值(不中断)。
pub fn init_from_disk() {
    {
        let mut a = lock();
        if let Ok(text) = std::fs::read_to_string(SETTINGS_FILE) {
            if let Ok(s) = serde_json::from_str::<Settings>(&text) {
                a.settings = s;
            } else {
                tracing::warn!("settings.json 解析失败,使用默认设置");
            }
        }
        if let Ok(text) = std::fs::read_to_string(DATA_FILE) {
            if let Ok(d) = serde_json::from_str::<DataFile>(&text) {
                a.data = d;
            } else {
                tracing::warn!("data.json 解析失败,使用空数据");
            }
        }
    }
}

pub fn save_settings() {
    let text = {
        let a = lock();
        serde_json::to_string_pretty(&a.settings).unwrap_or_else(|_| "{}".into())
    };
    if let Err(e) = std::fs::write(SETTINGS_FILE, text) {
        tracing::warn!("保存 settings.json 失败: {e}");
    }
}

pub fn save_data() {
    let text = {
        let a = lock();
        serde_json::to_string_pretty(&a.data).unwrap_or_else(|_| "{}".into())
    };
    if let Err(e) = std::fs::write(DATA_FILE, text) {
        tracing::warn!("保存 data.json 失败: {e}");
    }
}

pub fn set_status(msg: &str) {
    let mut a = lock();
    a.status = msg.to_string();
    a.status_at = crate::dates::unix_now();
    tracing::info!("[status] {msg}");
}

/// 按 (date, model) upsert 一行;返回是否替换了既有行。
pub fn upsert_daily(date: &str, row: DayModelUsage) -> bool {
    let mut a = lock();
    let rows = a.data.days.entry(date.to_string()).or_default();
    match rows.iter_mut().find(|r| r.model == row.model) {
        Some(existing) => {
            *existing = row;
            true
        }
        None => {
            rows.push(row);
            false
        }
    }
}

/// 清理超过 RETAIN_DAYS 的历史数据。
pub fn prune() {
    let cutoff_days = crate::dates::unix_now().div_euclid(86_400) - RETAIN_DAYS;
    let (cy, cm, cd) = crate::dates::civil_from_days(cutoff_days);
    let cutoff = format!("{cy:04}-{cm:02}-{cd:02}");
    let mut a = lock();
    let before = a.data.days.len();
    a.data
        .days
        .retain(|date, _| date.as_str() >= cutoff.as_str());
    let after = a.data.days.len();
    if before != after {
        tracing::info!(
            "[prune] 清理 {} 天历史数据(保留 {after} 天)",
            before - after
        );
    }
}
