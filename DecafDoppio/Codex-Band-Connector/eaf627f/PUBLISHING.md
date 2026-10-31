# GitHub 与 AstroBox 发布

## 2026-10-21 展示与适配声明

- 当前适配小米手环 10、10 NFC、11 及 11 NFC，发布清单增加 xmb10、xmb10nfc，与原有 11 系列共用同一 1.3.1 RPK。
- 使用维护者指定的新头图，保留六张模拟器界面图，另增加两张艺术化宣传图。
- 默认下载包使用 dist 中的标准 release 包，不重新构建；版本仍为 1.3.1 / 13001。

头图引用 media/cover.png，预览列表保留六张原始模拟器截图并追加 media/promo-efficiency.png、media/promo-interface.png。旧封面记录如下，当前配置以上述文件为准。

## 2026-10-16 审核第 1、4 项修正候选

资源图标最终使用同设计 256×256 版本以满足 CreatorConsole 尺寸限制；头图采用维护者选定的 B 版浅色 3:2 封面，手环内图标不变。发布构建改为标准 `aiot release`，不再编译 debug 包后手动改写清单及重签。版本仍为 1.3.1 / 13001，沿用原签名身份。

第 1、4 项的初次独立导出位于 `.publication/review-fix-2026-10-16/`；补齐第 2 项模拟器截图后的完整导出位于 `.publication/github-sync-2026-10-16/`。其中 `astrobox/` 可供后续 CreatorConsole 手动提交，`github-release/` 含 RPK 与 SHA256SUMS.txt。旧 `.publication/` 和 downloads 中的旧包均保留。第 3 项 CreatorConsole 提交仍由用户手动完成，不在本次 GitHub 同步中自动提交。

```powershell
npm run release
python scripts/prepare-publication.py --rpk dist/com.local.codexband.release.1.3.1.rpk --output-dir .publication/github-sync-2026-10-16
```

构建前设置 README 中的外部签名路径。导出目录存在时脚本会拒绝覆盖，可使用另一个新目录。新候选的校验记录见 `REVIEW_FIX_2026-10-16.md`，不将旧包的真机验收记录套用到新包。以下为原发布记录，原有“不重新编译”限制由本次用户明确要求的标准 release 修正替代；历史哈希仍仅代表旧包。

本轮 GitHub 同步范围：main 的源码、头像、六张截图和说明；astrobox-release 的资源预览配置、头像、截图和标准 release RPK；v1.3.1 Release 的当前下载附件、校验文件和更新说明。旧 RPK 与旧校验附件保留为带 `before-review-fix` 名称的备份，不移动或重写原 v1.3.1 Git 标签；Release 正文记录修正源码提交。GitHub About 更新为当前功能介绍。平台上架状态须等待维护者手动重提及官方审核。

## main 分支

提交源码、锁定依赖、构建脚本、许可证与 media。不要提交 dist、downloads、node_modules、target、.local、.publication、私钥或连接配置。本机旧产物可以保留，首次暂存须复核忽略规则与全部暂存文件。

## GitHub Releases

以实际确认的源码提交创建 v1.3.1 标签，Release 使用 RELEASE_NOTES.md；附件使用 .publication/github-release 中的安装包与 SHA256SUMS.txt。不重新编译替换已验收 RPK。

```powershell
python scripts/prepare-publication.py
```

命令默认读取本机 downloads 中已验收 1.3.1 RPK。新克隆可用 --rpk 外部已验收包路径；脚本校验固定哈希。

## AstroBox 发布分支

脚本生成 .publication/astrobox，作为未来 astrobox-release 分支的独立文件树；不把它提交到 main。操作前记录 main 的源码提交与 Release 对应关系。使用单独 checkout/worktree 提交该目录内容，避免覆盖源码工作目录。

快应用分支根目录包含 manifest_v2.json、media 和 downloads。设备 xmb10、xmb10nfc、xmb11 和 xmb11nfc（小米手环 10、10 NFC、11 及 11 NFC），版本 1.3.1；既有真机验收记录为普通手环 11，固件 4.100.139；作者 DecafDoppio 绑定 AstroBox。当前申请在官方资源库 PR 中新增 tmp/decafdoppio/codex-band-connector/resource.csv 与 request.json（schema_version=1，mode=create），不直接修改 index_v2.csv。resource.csv 含表头和本资源一行，引用发布分支实际提交 SHA；官方工作流在审核合并后应用索引。标签建议 Codex;AI;开发工具。手动 API 提交不伪造 CreatorConsole 客户端版本。CreatorConsole 自动创建仓库的流程与现有两库不同，提交时检查生成目标，勿自动创建重复仓库。

发布分支已创建，快应用审核申请见 https://github.com/AstralSightStudios/AstroBox-Repo/pull/1358 。是否上线以官方审核及工作流结果为准。

官方资料：
- https://abox.run/docs/creator-tools/resource-management/plugin-upload
- https://abox.run/docs/creator-tools/resource-management/free-resources
- https://github.com/AstralSightStudios/AstroBox-Repo/blob/main/assets/docs/ResAdptV2.md


