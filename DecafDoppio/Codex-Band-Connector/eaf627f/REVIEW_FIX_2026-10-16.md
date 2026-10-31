# Connector 审核修正：第 1、4 项

后续更正：维护者重新核对审核原文为“头图”，早先将第 1 项理解为图标不准确。图标最终按 CreatorConsole 边长小于 500px 的要求使用同设计 256×256 版本。随后生成三张简化封面，维护者明确选择 B 版浅色方案并要求同步 GitHub；当前 media/cover.jpg 已采用该 1536×1024 封面。下文 1024×1024 描述为最初处理记录，不代表平台复审已通过。

日期：2026-10-16。仅修改 Connector 仓库；Bridge 未修改。未安装、未推送、未发送授权申请、未提交审核。

## 第 1 项：资源头像

`media/icon.png` 原为 96×96，现替换为项目自有同设计 1024×1024 原图，来源为开发项目 `assets/brand/icon-1024.png`。保留大面积的中心标识，无小字；不是将 96×96 缩略图放大。手环应用的 `src/common/icon.png` 未改变。第 2 项 cover/preview 均未改变。

## 第 4 项：标准 release 打包

旧包的 `manifest.json` 和 `manifest-watch.json` 实际均含 `versionCode: 13001`；不能将审核报错简单归因为源文件漏写字段。旧脚本调用 `aiot build` 后自行遍历资源、改写清单、生成摘要和重签，未使用标准 release 流程。

新脚本调用 AIoT-toolkit 2.0.5 的 `aiot release`，由工具完整生成、排序、摘要和签名 RPK，不再事后改写包体。`src/manifest.json` 的发布名称、版本与 `release.json` 对齐为已发布候选使用的 Codex Band Connector / 1.3.1 / 13001；未提高版本号。`quickapp.config.js` 通过官方 signRequest 接口读取仓库外既有签名文件，验证原证书指纹和密钥匹配，不生成新身份、不复制私钥。

官方命令说明：https://iot.mi.com/vela/quickapp/zh/tools/toolkit/start.html

## 验证

- `npm run release` 成功，编译模式为 production。
- 最终 RPK 两份清单均为 1.3.1 / 13001 / Codex Band Connector，与旧交付包一致；主清单仅 packageInfo 构建信息改变，设备清单内容一致。
- 对旧已验收 1.3.1 RPK 运行 `node scripts/verify-alignment.cjs .validation/review-fix/previous-1.3.1.rpk`：14 个控制器/原生调用/生命周期阶段一致；11 个页面场景的渲染树、点击目标、样式及默认值一致。
- common 内全部图片、音频资源字节一致；RPK 文件集合一致，未带入 PEM 或 source map。JS 因 production 编译而字节变化，未声称字节一致。
- 资源页头像 PNG 尺寸确认 1024×1024；预览图与封面未修改。
- `git diff --check` 通过。

## 本地交付

标准产物：`dist/com.local.codexband.release.1.3.1.rpk`。

独立导出：`.publication/review-fix-2026-10-16/astrobox/`；安装包位于其 `downloads/Codex-Band-Connector-1.3.1.rpk`。另有 `github-release/` 安装包副本及 SHA256SUMS.txt。旧导出与 downloads 中的旧验收包保留。

新 RPK SHA-256：`d267d7bcce487a0ec162f88b2fdc59f52dcf213fd6cdab9f71aebb1838276906`。

签名证书指纹：`C9:41:D2:00:81:7C:83:47:95:B2:A3:31:A7:6E:9D:0A:74:C0:14:53:89:88:41:F9:28:12:F8:6C:EA:C2:4C:05`。

本次通过本地构建与差分检查，尚未做新包真机验收，也未获平台重新审核结果。RELEASE_STATUS.json 保留旧包真机验收记录；不将其哈希或验收状态替换为新候选。

## 第 2 项截图与后续同步

随后按维护者要求建立独立模拟器测试目录并拍摄六张含外壳原图，页面和业务逻辑保留，仅在测试副本适配通信。当前仓库将旧 `media/preview.png` 移除，替换为 `media/screenshots/` 中六张真实渲染图，README 以每行三张并列展示，发布清单 preview 引用六张图片。截图样例不冒称 Codex 原生事件或真机验收；完整来源和校验见 MEDIA_SOURCES.md。

维护者后续明确要求将第 1、2、4 项同步 GitHub，包括 main、AstroBox 发布分支和标准 release 下载附件；本报告前文“未推送”描述的是首次本地修正时的状态。第 3 项仍由维护者手动使用 CreatorConsole 提交，本轮不更改官方审核申请。
