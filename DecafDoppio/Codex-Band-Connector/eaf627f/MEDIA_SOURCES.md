# 展示素材来源

media/icon.png 使用项目保留的同设计 256×256 正式图标版本（assets/brand/icon-256.png），满足 CreatorConsole 图标边长小于 500px 的要求。其高清母版为 assets/brand/icon-1024.png；未放大旧的 96×96 缩略图或改动手环内图标。维护者确认自有素材可公开分发。

media/screenshots/ 下 01 至 06 开头的六张 PNG 于 2026-10-16 从 AIoT IDE 自带截图菜单“有表盘”导出，包含当前模拟器皮肤，尺寸均为 272×563。原图直接复制，未裁剪、放大、拼贴或重新绘制。场景包括单条授权列表及详情、多条授权列表、问题列表及详情、授权与问题并列。校验值和设备信息见 media/screenshots/capture-records.json。

截图实际运行版本为独立的 Connector 1.3.1 测试副本；页面、控制器、资源及 Bridge 核心沿用发布源码，仅在测试副本适配模拟器通信。镜像 vela-pre-4.0、皮肤 xiaomi_band_10、屏幕 212×520。授权与问题内容为维护者明确要求的本地截图样例，不是 Codex 原生 hook 事件，不执行电脑命令，也不作为真机互联验收。测试通信适配和令牌不包含在正式 RPK 或本仓库中。

旧 AI 生成的界面集合图 media/preview.png 已从当前仓库文件树、README 和发布预览配置中移除。历史提交不重写。

2026-10-21 按维护者指定素材更新：media/cover.png 原样复制自 media/screenshots/头图.png；图内标题为 Codex Band Bridge，按维护者指定保留原图。media/promo-efficiency.png、media/promo-interface.png 分别原样复制自“效率随身，智能触达.png”和“智能手环界面宣传海报.png”。三张图片未裁剪、缩放或重绘。两张宣传图属于艺术化展示，不作为模拟器原始截图或真机验收证据。原有六张模拟器截图和 capture-records.json 保留不变。旧 media/cover.jpg 保留为历史素材，当前展示与发布清单均引用 cover.png。

旧 SVG 封面和旧桌面演示图保留在本机并由 .gitignore 排除，不进入 main 分支。
