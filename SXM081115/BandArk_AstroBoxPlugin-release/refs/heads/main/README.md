# BandArk_AstroBoxPlugin · 发布产物

**环上方舟配置器** —— AstroBox V2 插件（Rust → `wasm32-wasip2`）的**构建产物仓库**。
本仓库只放上架用的产物，**源码不在公开**（私有源码仓库：`SXM081115/BandArk_AstroBoxPlugin`）。

## 目录

| 文件 | 说明 |
|---|---|
| `index.txt` | 只有一行 `dist` —— 官方聚合 Action 按它找产物目录 |
| `dist/manifest.json` | 插件清单：`name` / `version` / `entry` / `icon` / `api_level` / `permissions` |
| `dist/arknightspoor_art_plugin.wasm` | `entry` 指向的产物（3.2 MB，release，不带调试 tab） |
| `dist/icon.png` | `icon` 指向的图标（342 KB） |
| `dist/环上方舟配置器.abp` | 本地导入用的打包件，非必需 |

> 聚合器只读 `manifest.json`（外加客户端按 `entry` / `icon` / `additional_files` 去同目录取文件），
> 所以任何要上架的文件都必须在 `manifest.json` 里声明。当前 `additional_files` 为空。

## 上架链路

官方仓库 `AstralSightStudios/AstroBox-NG-Plugin-Repo` 的 `index.txt` 里登记的是**本仓库的 raw 基地址**：

```text
https://raw.githubusercontent.com/SXM081115/BandArk_AstroBoxPlugin-release/refs/heads/main/
```

它的 Action 会读 `<基地址>index.txt` → 得到 `dist` → 拉 `dist/manifest.json` 汇总进 `index.json`，
客户端插件市场读的就是这份 `index.json`。

## 发新版本

1. 在源码仓库构建：`python scripts/build_dist.py --out dist --package`
2. 把 `dist/` 整个覆盖到本仓库（**记得递增 `manifest.json` 的 `version`**）
3. `git add -A && git commit -m "release: x.y.z" && git push`
4. 等官方仓库的下一次 Action（push 触发，或 4 小时定时）—— 之后市场里就是新版本

## 插件信息

| 项 | 值 |
|---|---|
| 名称 | 环上方舟配置器 |
| 版本 | 1.0.0 |
| API Level / WASI | 3 / 2 |
| 权限 | `network` · `device` · `interconnect` · `register_interconnect_recv` · `thirdpartyapp` |
| 功能 | 从 PRTS 抓取干员立绘与资料 → 打包成资源包 → 下发给手环端（含高清立绘 1024 档） |
