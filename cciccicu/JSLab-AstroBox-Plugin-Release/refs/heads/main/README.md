# JSLab Sync — AstroBox 插件分发源

JSLab Sync 2.0.1 是 AstroBox v2 的 JSLab 配套插件，用于与小米 Vela 手环上的
`icu.ccicc.jslab` 应用通信。需要 AstroBox API Level 3 或更高版本，插件使用 WASI Preview 2。

## 功能

- 浏览、新建、编辑、重命名、删除、上传与下载手环脚本；
- 多行源码编辑、分块确认、请求超时与断线处理；
- 移除原有 48 KiB 脚本上限，大脚本支持列表、读取、保存和上传；
- 支持源码接口的大请求与响应，其他云代理接口仍保留 64 / 128 KiB 保护；
- 支持声明 `scriptStorageProtocol: 1` 的手环编辑脚本 data/config JSON，旧手环继续同步源码；
- 接收手环文件变化通知；同名上传需确认覆盖；
- 为手环的云端请求提供 AstroBox 网络桥接。

字体更换入口当前显示“即将上线”。云端账户、配对、激活和 AI 由 JSLab Cloud 服务处理。

## 安装与使用

从 AstroBox 官方插件源安装 JSLab Sync，或下载本仓库的
[JSLab-Sync-2.0.1.abp](JSLab-Sync-2.0.1.abp) 并通过 AstroBox 的插件安装入口导入。
连接兼容的手环并安装 JSLab 应用后，在插件中选择设备和刷新文件列表。

本仓库维护分发文件及对应版本的源码归档。与本次构建一致的完整插件源码、
锁定依赖、协议说明和测试数据见
[JSLab-Sync-2.0.1-source.zip](JSLab-Sync-2.0.1-source.zip)。
开发仓库为 [cciccicu/JSLab](https://github.com/cciccicu/JSLab/tree/main/astrobox-plugin-sync)。
使用 Rust stable、`wasm32-wasip2` target，在归档的 `astrobox-plugin-sync/` 目录运行
`cargo test --offline --locked` 和 `./build.ps1`；首次构建需先获取 Cargo.lock 对应的依赖。

本次发布通过 25 项 Rust 测试，以及 AstroBox → Vela 实际模块的协议回放，覆盖
大脚本、数据持久化、冲突检测、断线处理和旧 v2 协议兼容性。
未连接设备或模拟器进行新的实机验收。

## 分发格式

根目录 `index.txt` 指向 `jslab-sync/`，该目录包含 manifest、PNG 图标与 WASM Component。
AstroBox 官方聚合源读取这些原始文件；不将本仓库的二进制文件复制到官方聚合源。
`SHA256SUMS` 记录插件文件与 ABP 的 SHA-256，方便检查下载完整性。

## 权限

manifest 声明 `device`、`interconnect`、`network`、`thirdpartyapp` 和
`register_interconnect_recv`，分别用于发现设备、传输文件、桥接云端网络请求、
启动 JSLab 应用和接收 Interconnect 消息。

## 许可证

JSLab Sync 以 GPL-3.0 分发，完整许可证见 [LICENSE](LICENSE)。
