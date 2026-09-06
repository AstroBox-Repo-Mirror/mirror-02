# dswatch-sync

AstroBox v2 插件:采集 DeepSeek **余额**与**用量**,按固定 60s 周期推送到手环快应用「DS Watch」(`com.dswatch.periodreminder`)。

数据经 Interconnect 推送到手环,与手环端 DS Watch 快应用 (`rufus-fan/dswatch`) 使用同一份快照协议
(`provider-usage-snapshot-v1`)。手环打开 DS Watch 应用时,插件会立即强推一版快照。

## 依赖

- Rust 1.85+ (edition 2024)
- `rustup target add wasm32-wasip2`
- Python 3(构建脚本)

## 构建

```bash
# Debug 构建到 dist/
python scripts/build_dist.py

# Release 构建并打包为 .abp(可导入 AstroBox 安装)
python scripts/build_dist.py --release --package
```

产物在 `dist/`:`manifest.json` + `dswatch_sync.wasm` + `icon.png`(+ `dswatch-sync.abp`)。

> 若首次 `cargo build` 拉取依赖失败,可配置镜像:项目根目录建 `.npmrc` 不适用,
> 改用 cargo 全局配置 `~/.cargo/config.toml` 指向镜像源(如
> `[source.crates-io] replace-with = 'rsproxy-sparse'`)。

## 安装与使用

1. AstroBox 中导入 `dist/dswatch-sync.abp`(或通过插件市场上架版本安装)。
2. 首次加载按提示授予 `network`、`device`、`interconnect`、`register_interconnect_recv`、`thirdpartyapp` 权限。
3. AstroBox 连接小米手环,并确认手环上已安装 DS Watch 快应用。
4. 打开插件页面,填入两个参数后点「保存设置」:
   - **DeepSeek API Key**:余额接口用,`platform.deepseek.com` 开放平台创建即可;
   - **平台 Token**:用量导出接口用,浏览器打开 `platform.deepseek.com`,F12 找到导出请求头 `Authorization: Bearer ...`。
5. 点「立即同步并推送」验证;此后每 60s 自动同步一次(余额/用量变化才推送,手环打开应用时立即推)。

## 参数

| 参数 | 必填 | 说明 |
| --- | --- | --- |
| DeepSeek API Key | 是 | 余额查询(`GET api.deepseek.com/user/balance`) |
| 平台 Token | 否 | 用量导出(`GET platform.deepseek.com/api/v0/usage/export`),不填则仅推送余额 |

推送周期 60s、目标包名、接口地址均已固定,无其它配置。

## 上架官方插件源

1. 把本项目推送到 GitHub,根目录需保留 `index.txt`(内容为 `dist`,即构建产物目录名)。
2. 向 [AstroBox-NG-Plugin-Repo](https://github.com/AstralSightStudios/AstroBox-NG-Plugin-Repo)
   的 `index.txt` 末尾追加一行你的 raw 基地址:
   `https://raw.githubusercontent.com/<owner>/<repo>/refs/heads/<branch>/`
   (注意保留末尾斜杠)。
3. 提 PR,注明插件名、API Level 2 / WASI Preview 2、权限清单。

## 权限清单

`network`(WASI HTTP)、`device`、`interconnect`、`register_interconnect_recv`、`thirdpartyapp`。

## 目录结构

```
dswatch-sync/
├── Cargo.toml / manifest.json / index.txt / icon.png
├── scripts/build_dist.py      # 构建与打包
├── src/
│   ├── lib.rs                 # 插件入口(生命周期/事件分发)
│   ├── engine.rs              # 定时器、设备发现、互联注册、同步与推送
│   ├── deepseek.rs            # 余额 + 用量导出 HTTP 接口
│   ├── import.rs              # 用量 CSV zip 导入聚合
│   ├── snapshot.rs            # provider-usage-snapshot-v1 快照构建
│   ├── state.rs               # settings/data 持久化与全局状态
│   ├── ui.rs                  # 设置页(API Key + 平台 Token)
│   ├── dates.rs               # 日期工具
│   └── logger.rs              # 日志初始化
```

## License

[MIT](LICENSE)
