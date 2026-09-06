//! dswatch-sync - AstroBox v2 插件入口(wasm32-wasip2 Component)。
//!
//! 采集 DeepSeek 余额与用量,按固定 60s 周期推送到「DS Watch」手环快应用
//! (`com.dswatch.periodreminder`)。
//!
//! 注意:HTTP 动作(查余额/导出)是阻塞式的,执行期间该插件的事件分发会暂停。

use astrobox_ng_wit::exports::astrobox::psys_plugin::{
    event::{self, EventType},
    lifecycle,
};
use astrobox_ng_wit::FutureReader;

mod dates;
mod deepseek;
mod engine;
mod import;
mod logger;
mod snapshot;
mod state;
mod ui;

struct Plugin;

/// 返回一个"已完成"的 FutureReader<String>
fn immediate_string(value: String) -> FutureReader<String> {
    let (writer, reader) = astrobox_ng_wit::wit_future::new::<String>(|| "".to_string());
    astrobox_ng_wit::spawn(async move {
        let _ = writer.write(value).await;
    });
    reader
}

/// 返回一个"已完成"的 FutureReader<()>
fn immediate_unit() -> FutureReader<()> {
    let (writer, reader) = astrobox_ng_wit::wit_future::new::<()>(|| ());
    astrobox_ng_wit::spawn(async move {
        let _ = writer.write(()).await;
    });
    reader
}

impl lifecycle::Guest for Plugin {
    #[allow(async_fn_in_trait)]
    fn on_load() -> () {
        logger::init();
        astrobox_ng_wit::block_on(async {
            engine::init().await;
        });
        tracing::info!("dswatch-sync 插件加载完成");
    }
}

impl event::Guest for Plugin {
    #[allow(async_fn_in_trait)]
    fn on_event(event_type: EventType, event_payload: String) -> FutureReader<String> {
        tracing::info!("[entry] on_event type={event_type:?} payload={event_payload}");
        astrobox_ng_wit::block_on(engine::handle_event(event_type, &event_payload));
        immediate_string(String::new())
    }

    #[allow(async_fn_in_trait)]
    fn on_ui_event(
        event_id: String,
        event: event::Event,
        event_payload: String,
    ) -> FutureReader<String> {
        tracing::info!("[entry] on_ui_event id={event_id} ev={event:?} payload={event_payload}");
        // 同步处理:block_on 驱动其中的宿主调用,完成后立即重绘
        astrobox_ng_wit::block_on(ui::handle_ui_event(&event_id, &event, &event_payload));
        immediate_string(String::new())
    }

    #[allow(async_fn_in_trait)]
    fn on_ui_render(element_id: String) -> FutureReader<()> {
        tracing::info!("[entry] on_ui_render id={element_id}");
        ui::render_page(&element_id);
        immediate_unit()
    }

    #[allow(async_fn_in_trait)]
    fn on_card_render(_card_id: String) -> FutureReader<()> {
        // 未注册设备详情页卡片,无操作
        immediate_unit()
    }
}

astrobox_ng_wit::export!(Plugin);
