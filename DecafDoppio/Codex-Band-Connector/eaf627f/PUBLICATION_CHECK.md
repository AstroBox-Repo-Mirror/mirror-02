# 发布整理复核 — 2026-10-15

- main 的忽略规则排除 dist、downloads、build、node_modules、.publication、旧封面源文件及旧演示图；源码和必要构建输入保留。
- 新媒体文件与维护者提供文件逐字节一致。作者、版权署名和计划仓库链接配置为 DecafDoppio。
- 手环运行源码与私人工作区最新 release-alignment/watch/src 逐字节一致，本次未改应用代码、资源或签名。
- 已验收 RPK 原样用于 Releases 和 AstroBox 文件树，SHA-256：`4a672f0a248f8683b0fdccf7fddede3de672ba289f11aea035bb9f9825d84b0d`。
- 发布导出命令已执行通过；AstroBox manifest 引用的封面、图标、预览图和设备包均存在。
- 两库拟公开文件与发布产物共检查 3524 个文件/归档条目，未发现匹配的私钥、访问令牌或本机用户目录路径；模式检查不保证识别所有形式的秘密，首次提交仍须复核暂存清单。
- Python 脚本语法与源码 JSON 解析通过。本次未重跑源码编译或完整历史回归；此前独立构建及历史测试结果见 VALIDATION.md。
- 当前仅生成本地候选；本地 Git 已初始化；维护者提供了 DecafDoppio 名下两个远程仓库地址。尚未创建发布分支、上传 Release 或提交 AstroBox 审核。

发布步骤见 PUBLISHING.md。原包、旧图和私人审计证据保留，不作为 main 的公开文件。
