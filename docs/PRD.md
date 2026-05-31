# PRD：CalGuard — 基于 Next.js + React + Tauri + Rust 的本地优先日程健康诊断桌面应用

> 文档版本：v0.3  
> 文档日期：2026-05-31  
> 目标读者：产品、设计、前端、Rust 后端、个人开发者  
> 产品阶段：MVP 设计稿  
> 技术方向：Next.js + React + TypeScript + Tauri + Rust + SQLite  
> 明确排除：Vue、纯 Web SaaS、完整日历客户端、日历写回、AI 自动排期

---

## 0. 本版 PRD 的关键决策

本 PRD 已基于“我要使用 Next.js + React，不使用 Vue”的要求重新收敛。

### 0.1 最终产品形态

CalGuard 是一个 **桌面优先、本地优先、只读优先** 的日程健康诊断应用。

它不是 Google Calendar、Outlook、Apple Calendar 的替代品，而是读取已有日历数据，分析未来一段时间内的：

- 日程冲突
- 专注时间不足
- 会议过载
- 碎片化时间
- 晚间会议
- 连续会议
- 工作日边界被侵蚀
- 多日历之间的隐性冲突

### 0.2 技术栈决策

| 层级 | 决策 |
|---|---|
| 桌面容器 | Tauri v2 |
| UI 框架 | Next.js + React + TypeScript |
| 前端渲染方式 | Static Export，不依赖 Node.js 服务端 |
| Rust 核心 | 日历解析、时间计算、冲突检测、健康评分、本地存储 |
| 本地数据库 | SQLite |
| MVP 日历源 | 本地 `.ics` 文件、远程 `.ics` URL |
| MVP 不做 | Vue、SSR、API Routes、Server Actions、CalDAV、OAuth、日历写回、AI 自动修改 |

### 0.3 最重要的技术约束

Tauri 中使用 Next.js 时，Next.js 必须作为静态前端构建产物运行。也就是说：

- 必须使用 `output: 'export'`。
- Tauri 的 `frontendDist` 指向 Next.js 构建后的 `out` 目录。
- MVP 不依赖 Next.js API Routes、SSR、Server Actions、Middleware、动态服务端路由。
- 所有需要系统能力、文件访问、数据库、网络拉取远程 ICS 的功能，都通过 Tauri command 调用 Rust 实现。

这不是限制，而是架构边界：

```text
Next.js + React：负责 UI、交互、状态展示
Tauri：负责桌面壳、权限、前后端桥接
Rust：负责核心业务逻辑、文件、SQLite、网络、分析算法
```

---

## 1. 一句话产品描述

CalGuard 是一款本地优先的桌面应用，用于聚合本地或远程 ICS 日历，自动诊断日程冲突、会议过载、专注时间不足和时间碎片化，并生成可解释的日程健康报告。

---

## 2. 产品定位

### 2.1 产品定位语

```text
A local-first calendar health analyzer built with Rust, Tauri, Next.js and React.
```

中文定位：

```text
一个本地优先的日程健康诊断工具。
```

### 2.2 产品不是

CalGuard 不是：

- 不是完整日历 App。
- 不是 Calendly 替代品。
- 不是 Reclaim / Motion / Morgen 这类完整 AI 日程规划工具。
- 不是团队协作 SaaS。
- 不是任务管理器。
- 不是默认会修改用户日历的工具。

### 2.3 产品是

CalGuard 是：

- 日程冲突检测器。
- 会议健康分析器。
- 专注时间发现工具。
- 多日历只读聚合分析器。
- 本地优先隐私工具。
- Rust 学习与产品实践结合的中等复杂度项目。

---

## 3. 背景与机会

### 3.1 用户问题

很多日历应用只回答一个问题：

```text
我接下来有什么安排？
```

但用户真正还关心：

```text
我的日程是否健康？
我这一周还有没有连续专注时间？
哪些会议冲突了？
哪一天会议过载？
哪些会议把我的一天切碎了？
我的工作日是否被延伸到晚上？
我的工作日历和私人日历有没有冲突？
```

现有日历工具多强调“创建、同步、提醒、共享、预约”，但对“时间质量诊断”的关注不足。

### 3.2 产品机会

CalGuard 不与大型日历应用正面竞争，而是做一个更窄的诊断层：

```text
已有日历数据
→ 本地读取
→ 本地分析
→ 可视化问题
→ 给出解释
→ 导出报告
→ 用户自行决策
```

### 3.3 为什么适合用 Rust 学习

该项目可以系统练习 Rust 的核心能力：

- 文件读取
- 网络请求
- 错误处理
- 时间与时区处理
- iCalendar 格式解析
- 重复事件展开
- 区间算法
- SQLite 数据持久化
- Tauri command
- 跨平台桌面打包
- 测试与 fixture 设计

---

## 4. MVP 原则

MVP 的目标不是做一个“大而全”的日历产品，而是做一个可以完整闭环的诊断产品。

### 4.1 MVP 必须满足

MVP 必须做到：

1. 用户能导入至少一个本地 `.ics` 文件。
2. 用户能添加一个远程 `.ics` URL。
3. 用户能看到未来 7 / 14 / 30 天的日程健康分析。
4. 用户能看到冲突列表。
5. 用户能看到空闲时间和专注时间块。
6. 用户能看到健康分数和扣分原因。
7. 用户能导出 Markdown / JSON 报告。
8. 用户的数据默认保存在本地。
9. 应用不要求注册账号。
10. 应用不写回用户日历。

### 4.2 MVP 必须克制

MVP 不做：

- CalDAV。
- Google OAuth。
- Outlook OAuth。
- Apple Calendar 深度集成。
- 日历写回。
- 创建、编辑、删除日历事件。
- AI 自动重排。
- 团队协作。
- 移动端。
- Web SaaS。
- 复杂主题系统。
- 自然语言创建日程。
- 会议预约链接。

### 4.3 MVP 成功定义

MVP 成功不是“功能多”，而是：

```text
用户导入日历后，能在 3 分钟内发现至少一个有价值的时间问题。
```

---

## 5. 目标用户

## 5.1 用户画像 A：会议密集型知识工作者

| 项目 | 描述 |
|---|---|
| 身份 | 工程师、产品经理、设计师、咨询顾问、团队负责人 |
| 痛点 | 会议多、连续工作时间少、日程被切碎 |
| 动机 | 想知道本周是否还有深度工作时间 |
| 场景 | 周一早上查看本周日程健康报告 |
| 价值 | 提前发现过载日和冲突会议 |

## 5.2 用户画像 B：多日历用户

| 项目 | 描述 |
|---|---|
| 身份 | 同时使用公司日历、个人日历、订阅日历的用户 |
| 痛点 | 多个日历之间不可见，容易产生隐性冲突 |
| 动机 | 想在本地聚合分析，而不是把所有日历交给云服务 |
| 场景 | 导入工作 ICS 和个人 ICS，检查未来 14 天冲突 |
| 价值 | 保护隐私的同时发现冲突 |

## 5.3 用户画像 C：隐私敏感用户

| 项目 | 描述 |
|---|---|
| 身份 | 开发者、独立研究者、律师、医疗/金融从业者 |
| 痛点 | 不希望完整日历数据上传第三方 SaaS |
| 动机 | 本地分析、本地缓存、可控导出 |
| 场景 | 使用本地 `.ics` 文件进行离线分析 |
| 价值 | 不注册、不上传、不写回 |

## 5.4 用户画像 D：技术用户和开源用户

| 项目 | 描述 |
|---|---|
| 身份 | 开发者、Rust 学习者、自动化爱好者 |
| 痛点 | 想要可验证、可扩展、可读源码的日程分析工具 |
| 动机 | 希望通过真实项目学习 Rust + Tauri + Next.js |
| 场景 | Fork 项目、阅读核心算法、贡献功能 |
| 价值 | 技术栈清晰，模块边界清楚 |

---

## 6. 核心用户旅程

## 6.1 首次使用流程

```text
打开应用
→ 看到产品说明和隐私承诺
→ 选择导入方式
   → 本地 .ics 文件
   → 远程 .ics URL
→ 设置工作时间
   → 默认工作日：周一至周五
   → 默认工作时间：09:00-18:00
   → 默认专注块：90 分钟
→ 点击“生成分析”
→ 进入 Dashboard
→ 查看冲突、专注时间、健康分数
→ 导出报告或关闭应用
```

## 6.2 日常使用流程

```text
打开应用
→ 自动读取本地缓存
→ 用户手动点击“刷新远程日历”
→ Dashboard 更新
→ 用户查看 Top Risks
→ 用户进入 Conflicts 或 Free Time 页面
→ 用户导出本周报告
```

## 6.3 用户价值闭环

```text
导入日历
→ 发现问题
→ 理解原因
→ 获得建议
→ 导出或手动调整日程
→ 下次重新分析
```

---

## 7. 功能优先级

## 7.1 P0：MVP 必须做

| 模块 | 功能 | 是否 P0 |
|---|---|---:|
| Onboarding | 首次导入本地 ICS | 是 |
| Onboarding | 添加远程 ICS URL | 是 |
| Calendar Sources | 日历源列表、启用、禁用、删除 | 是 |
| Parser | 解析基础 VEVENT | 是 |
| Parser | 支持基本 RRULE 展开 | 是 |
| Analysis | 冲突检测 | 是 |
| Analysis | 空闲时间计算 | 是 |
| Analysis | 专注时间块识别 | 是 |
| Analysis | 过载日检测 | 是 |
| Analysis | 健康分数 | 是 |
| UI | Dashboard | 是 |
| UI | Conflicts 页面 | 是 |
| UI | Free Time 页面 | 是 |
| UI | Settings 页面 | 是 |
| Export | Markdown 报告 | 是 |
| Export | JSON 报告 | 是 |
| Privacy | 隐私模式导出 | 是 |
| Storage | SQLite 本地缓存 | 是 |
| Error | 用户可读错误提示 | 是 |

## 7.2 P1：MVP 后增强

| 模块 | 功能 | 是否 P1 |
|---|---|---:|
| CLI | `calguard analyze` 命令 | 是 |
| Sync | 自动刷新远程 ICS | 是 |
| Analysis | 更细的碎片化评分 | 是 |
| UI | 周视图 timeline 交互增强 | 是 |
| Export | 报告模板配置 | 是 |
| Settings | 自定义评分权重 | 是 |
| Security | 系统 keychain 存储凭据 | 是 |

## 7.3 P2：长期功能

| 模块 | 功能 | 是否 P2 |
|---|---|---:|
| CalDAV | 只读 CalDAV 同步 | 是 |
| OAuth | Google / Outlook OAuth | 是 |
| Write-back | 可选写回 Focus Block | 是 |
| AI | 自然语言解释和建议 | 是 |
| Team | 团队日程健康分析 | 是 |
| Mobile | 移动端 | 是 |

---

## 8. 功能详细需求

## 8.1 Onboarding

### 目标

让用户在首次启动后 2-3 分钟内完成第一个日历导入并看到分析结果。

### 页面内容

1. 产品说明。
2. 隐私承诺。
3. 导入方式选择。
4. 工作时间设置。
5. 分析范围选择。
6. 生成第一份报告。

### 表单字段

| 字段 | 默认值 | 必填 | 说明 |
|---|---|---:|---|
| 日历名称 | `My Calendar` | 是 | 用户可编辑 |
| 导入方式 | 本地 ICS | 是 | 本地文件 / 远程 URL |
| 本地文件路径 | 无 | 条件必填 | 选择 `.ics` 文件 |
| 远程 URL | 无 | 条件必填 | 仅支持 HTTP/HTTPS，MVP 建议 HTTPS |
| 分析范围 | 14 天 | 是 | 7 / 14 / 30 天 |
| 工作日 | 周一至周五 | 是 | 可多选 |
| 工作时间 | 09:00-18:00 | 是 | 用于计算空闲块 |
| 专注时间目标 | 90 分钟 | 是 | 用于 Deep Work 识别 |

### 验收标准

- 用户可以跳过注册账号。
- 用户可以导入本地 `.ics`。
- 用户可以输入远程 `.ics` URL。
- 导入失败时显示明确错误。
- 导入成功后自动进入 Dashboard。
- 未联网时仍可使用本地 `.ics`。

---

## 8.2 Calendar Sources 日历源管理

### 目标

用户可以管理参与分析的日历源。

### MVP 支持的日历源

```text
1. 本地 .ics 文件
2. 远程 .ics URL
```

### 日历源字段

| 字段 | 类型 | 说明 |
|---|---|---|
| id | string | UUID |
| name | string | 用户自定义名称 |
| kind | enum | `local_ics_file` / `remote_ics_url` |
| color | string | UI 颜色标识 |
| enabled | boolean | 是否参与分析 |
| lastSyncedAt | datetime/null | 最近同步时间 |
| syncStatus | enum | idle / syncing / success / failed |
| errorMessage | string/null | 同步失败原因 |

### 操作

- 添加本地 ICS。
- 添加远程 ICS URL。
- 启用 / 禁用日历源。
- 删除日历源。
- 手动刷新远程日历。
- 查看最近同步状态。

### 验收标准

- 禁用某个日历源后，Dashboard、Conflicts、Free Time 的分析结果应更新。
- 删除日历源后，其事件不再参与分析。
- 远程同步失败时，保留上一次成功缓存的数据。
- 同步失败不应导致整个应用不可用。

---

## 8.3 ICS 解析

### 目标

将 `.ics` 文件中的事件解析为 CalGuard 内部模型。

### MVP 解析对象

MVP 只解析 `VEVENT`。

暂不解析：

- `VTODO`
- `VJOURNAL`
- `VALARM`
- 复杂参与人状态
- 复杂会议邀请流程

### MVP 支持字段

| ICS 字段 | 内部字段 | P0 |
|---|---|---:|
| UID | uid | 是 |
| SUMMARY | title | 是 |
| DTSTART | startsAt | 是 |
| DTEND | endsAt | 是 |
| DURATION | duration | 是，若无 DTEND 时使用 |
| LOCATION | location | 是 |
| DESCRIPTION | description | 是 |
| STATUS | status | 是 |
| TRANSP | transparency | 是 |
| RRULE | recurrenceRule | 是，基础支持 |
| EXDATE | excludedDates | 是，基础支持 |
| RECURRENCE-ID | recurrenceId | P1 |
| ATTENDEE | attendees | P2 |
| ORGANIZER | organizer | P2 |

### 状态处理

| ICS 状态 | 处理 |
|---|---|
| CONFIRMED | 参与 busy 计算 |
| TENTATIVE | 默认参与 busy，用户可配置 |
| CANCELLED | 不参与 busy 计算 |

### 透明事件处理

| TRANSP | 处理 |
|---|---|
| OPAQUE | 占用时间 |
| TRANSPARENT | 默认不占用时间 |
| 缺省 | 按 OPAQUE 处理 |

### 验收标准

- 能解析普通单次事件。
- 能解析全天事件。
- 能解析跨天事件。
- 能识别取消事件。
- 能识别透明事件。
- 能解析基础 RRULE。
- 单个事件解析失败不应导致整个文件失败。
- 解析失败应返回行号或上下文信息。
- 至少准备 20 个 fixture 文件覆盖常见 ICS 情况。

---

## 8.4 重复事件展开

### 目标

将重复事件在指定分析窗口内展开为具体事件实例。

### 输入

```rust
RecurringEvent {
    uid,
    title,
    starts_at,
    ends_at,
    recurrence_rule,
    excluded_dates,
}
```

### 输出

```rust
EventInstance {
    id,
    event_id,
    source_id,
    uid,
    title,
    starts_at,
    ends_at,
    is_all_day,
    status,
    transparency,
}
```

### 规则

- 只展开分析窗口内的实例。
- 每个重复事件最多展开 1000 个实例。
- 无结束条件的重复规则必须受分析窗口限制。
- `EXDATE` 应排除对应实例。
- `CANCELLED` 不参与 busy。
- 全天事件保留 `is_all_day = true`。

### 验收标准

- 每周重复会议能正确展开。
- 每日重复会议能正确展开。
- 带 `COUNT` 的重复会议能正确停止。
- 带 `UNTIL` 的重复会议能正确停止。
- `EXDATE` 能排除实例。
- 不会因无限重复规则导致死循环。

---

## 8.5 冲突检测

### 目标

检测多个日历事件之间的时间重叠。

### 冲突定义

两个事件 A 和 B 满足以下条件时视为冲突：

```text
A.starts_at < B.ends_at && B.starts_at < A.ends_at
```

以下情况不视为冲突：

- A 的结束时间等于 B 的开始时间。
- B 的结束时间等于 A 的开始时间。
- 任一事件为 `TRANSPARENT`。
- 任一事件为 `CANCELLED`。
- 任一事件来自禁用日历源。
- 用户已忽略该冲突。

### 严重级别

| 级别 | 条件 |
|---|---|
| Low | 重叠 1-10 分钟 |
| Medium | 重叠 10-30 分钟 |
| High | 重叠 30 分钟以上 |
| Critical | 三个及以上事件在同一时间段重叠 |

### 页面展示字段

| 字段 | 说明 |
|---|---|
| 时间 | 冲突发生时间段 |
| 严重级别 | Low / Medium / High / Critical |
| 事件列表 | 参与冲突的事件 |
| 日历源 | 每个事件所属日历 |
| 重叠时长 | 冲突持续多久 |
| 建议 | 可考虑移动或忽略 |

### 验收标准

- 能检测同一日历内冲突。
- 能检测不同日历之间冲突。
- 能正确处理边界相接事件。
- 能按严重级别过滤。
- 能按日历源过滤。
- 能忽略单个冲突。
- 被忽略冲突不再影响健康分数。

---

## 8.6 空闲时间计算

### 目标

计算用户工作时间内的空闲时间块。

### 用户设置

| 设置 | 默认值 |
|---|---|
| 工作日 | 周一至周五 |
| 工作时间 | 09:00-18:00 |
| 午休时间 | 12:00-13:00 |
| 最小专注块 | 90 分钟 |
| 最小短空闲块 | 30 分钟 |
| 是否包含全天事件 | 默认不阻塞整天 |
| 是否包含透明事件 | 默认不包含 |

### 空闲块类型

| 类型 | 条件 |
|---|---|
| DeepWork | 大于等于用户设置的专注块时长，默认 90 分钟 |
| ShortGap | 30-89 分钟 |
| MicroGap | 小于 30 分钟 |
| Lunch | 与用户设置午休时间重合 |
| OutsideWork | 工作时间外，不作为核心指标 |

### 示例

```text
工作时间：09:00-18:00
Busy：
10:00-11:00
13:00-14:30
16:00-16:30

Free：
09:00-10:00 ShortGap
11:00-13:00 DeepWork
14:30-16:00 DeepWork
16:30-18:00 DeepWork
```

### 验收标准

- 能按天计算空闲时间。
- 能按周汇总 Deep Work 总时长。
- 能展示每天最长空闲块。
- 能识别无专注时间的日期。
- 能根据用户工作时间设置动态更新结果。

---

## 8.7 日程健康分数

### 目标

用一个 0-100 的分数表达用户当前分析周期内的日程健康程度。

### 初始模型

```text
基础分：100
最终分数限制在 0-100。
```

### 扣分项

| 项目 | 扣分 |
|---|---:|
| Critical 冲突 | 每个 -15 |
| High 冲突 | 每个 -10 |
| Medium 冲突 | 每个 -5 |
| Low 冲突 | 每个 -2 |
| 每天无 Deep Work 块 | 每天 -8 |
| 每天会议总时长 > 5 小时 | 每天 -8 |
| 每天会议总时长 > 7 小时 | 每天额外 -7 |
| 19:00 后会议 | 每个 -5 |
| 连续会议超过 3 个 | 每组 -5 |
| 午休被会议覆盖 | 每天 -5 |
| MicroGap 超过 3 个 | 每天 -3 |

### 加分项

| 项目 | 加分 |
|---|---:|
| 分析周期内无冲突 | +8 |
| 每个工作日都有 Deep Work 块 | +8 |
| 至少一天无会议 | +5 |
| 每天会议均少于 4 小时 | +5 |

### 输出

```rust
HealthScore {
    score: u8,
    grade: HealthGrade,
    positive_reasons: Vec<ScoreReason>,
    negative_reasons: Vec<ScoreReason>,
}
```

### 分数等级

| 分数 | 等级 |
|---:|---|
| 90-100 | Excellent |
| 75-89 | Good |
| 60-74 | Warning |
| 40-59 | Poor |
| 0-39 | Critical |

### 验收标准

- 分数稳定，同样输入产生同样输出。
- 每个扣分项必须有可解释原因。
- 点击原因能定位到对应日期或事件。
- 被忽略的冲突不影响分数。
- 用户修改设置后分数应重新计算。

---

## 8.8 专注时间分析

### 目标

帮助用户判断本周是否有足够的连续深度工作时间。

### 指标

| 指标 | 说明 |
|---|---|
| Deep Work Blocks | 深度工作块数量 |
| Focus Hours | 深度工作块总时长 |
| Longest Free Block | 最长连续空闲时间 |
| No Focus Days | 没有深度工作块的工作日 |
| Meeting Density | 工作时间内会议占比 |
| Fragmentation Score | 时间碎片化程度 |

### Fragmentation Score 初版

```text
Fragmentation Score = 0-100
分数越高表示越碎片化。

影响因素：
- MicroGap 数量
- 连续会议数量
- 最长空闲块长度
- 会议分布跨度
```

### 验收标准

- 能展示每天 Deep Work 数量。
- 能展示本周 Focus Hours。
- 能展示最长空闲块。
- 能识别 No Focus Days。
- 能展示 Meeting Density。

---

## 8.9 过载日检测

### 目标

识别会议过多、专注时间不足或工作时间被侵蚀的日期。

### 默认规则

| 条件 | 状态 |
|---|---|
| 会议总时长 > 5 小时 | Overloaded |
| 会议总时长 > 7 小时 | Severely Overloaded |
| 会议数量 > 8 个 | Overloaded |
| 连续会议超过 3 个 | Risk |
| 午休时间被覆盖 | Risk |
| 没有 Deep Work 块 | Risk |
| 19:00 后仍有会议 | Boundary Risk |

### 验收标准

- Dashboard 能展示未来 7 天的过载状态。
- 每个过载日都有具体原因。
- 用户能点击日期进入详情。
- 用户可以调整过载阈值。

---

## 8.10 修复建议

### MVP 定位

MVP 只做“建议”，不做“自动修改”。

### 建议类型

| 类型 | 说明 | MVP |
|---|---|---:|
| MoveEvent | 建议移动某个事件 | 是，文本建议 |
| AddFocusBlock | 建议保护某个空闲块 | 是，文本建议 |
| AddBuffer | 建议连续会议之间增加缓冲 | 是，文本建议 |
| DeclineOrShortenMeeting | 建议拒绝或缩短会议 | P1 |
| MergeMeetings | 建议合并会议 | P1 |
| WriteBackFocusBlock | 写回日历创建专注块 | P2 |

### 示例

```text
建议：在周二 13:30-15:00 保护一段 Focus Time。
原因：这是当天唯一一个超过 90 分钟的空闲时间块。
```

```text
建议：检查周三 10:30 的 Candidate Interview。
原因：它与 Product Review 重叠 30 分钟，冲突级别为 High。
```

### 验收标准

- 每条建议必须有原因。
- 每条建议必须关联日期、事件或空闲块。
- 用户可以复制建议文本。
- 用户可以忽略建议。
- MVP 不写回任何日历。

---

## 8.11 报告导出

### 目标

用户可以将分析结果保存为 Markdown 或 JSON。

### 报告格式

MVP 支持：

- Markdown
- JSON

### Markdown 报告结构

```md
# Calendar Health Report

Period: 2026-06-01 to 2026-06-14
Generated at: 2026-05-31 10:00

## Summary

Score: 72 / 100
Grade: Warning
Conflicts: 3
Deep Work Hours: 8.5h
Overloaded Days: 2

## Top Risks

1. Wednesday has 2 high-severity conflicts.
2. Thursday has no deep work block.
3. Friday has 6.5h of meetings.

## Conflicts

- 2026-06-03 10:30-11:00
  - Product Review
  - Candidate Interview

## Free Time

- Monday 13:30-15:30 Deep Work
- Tuesday 09:00-10:00 Short Gap

## Suggestions

1. Move Candidate Interview away from Wednesday 10:30.
2. Protect Tuesday 13:30-15:00 as Focus Time.
```

### 隐私导出模式

用户可选择：

| 选项 | 说明 |
|---|---|
| 包含事件标题 | 默认开启 |
| 隐藏事件标题 | 用 `Busy Event` 替代 |
| 包含地点 | 默认关闭 |
| 包含日历源名称 | 默认开启 |
| 包含描述 | 默认关闭 |

### 验收标准

- 能导出 Markdown。
- 能导出 JSON。
- 导出前弹出隐私提醒。
- 隐私模式不包含事件标题、地点、描述。
- JSON schema 包含版本号。

---

## 9. 页面与交互设计

## 9.1 Next.js 路由设计

由于应用运行在 Tauri 中并使用静态导出，路由应保持简单。

推荐使用 App Router，但所有页面都应能静态导出。

```text
/app
  /page.tsx                    -> 启动页，根据状态跳转 Dashboard 或 Onboarding
  /onboarding/page.tsx          -> 首次导入
  /dashboard/page.tsx           -> 总览
  /sources/page.tsx             -> 日历源管理
  /conflicts/page.tsx           -> 冲突列表
  /free-time/page.tsx           -> 空闲时间
  /focus/page.tsx               -> 专注时间分析
  /reports/page.tsx             -> 报告导出
  /settings/page.tsx            -> 设置
```

### 路由约束

MVP 不使用：

- `route.ts` 作为 API Routes。
- Server Actions。
- SSR。
- `getServerSideProps`。
- Middleware。
- Next.js 后端鉴权。

---

## 9.2 Dashboard 页面

### 页面目标

让用户一眼看出当前分析周期内的日程健康情况。

### 页面模块

| 区域 | 内容 |
|---|---|
| 顶部 | 分析范围、刷新按钮、导出按钮 |
| 指标卡 | 健康分数、冲突数量、Deep Work 总时长、过载天数 |
| 周概览 | 每天会议时长、空闲块、风险标记 |
| Top Risks | 最重要的 5 个风险 |
| Suggestions | 最重要的 5 条建议 |

### 线框

```text
┌──────────────────────────────────────────────────────────────┐
│ CalGuard                         Range: Next 14 Days          │
├──────────────────────────────────────────────────────────────┤
│ Score 72   Conflicts 3   Focus 8.5h   Overloaded Days 2      │
├──────────────────────────────┬───────────────────────────────┤
│ Weekly Overview              │ Top Risks                     │
│                              │ 1. Wed conflict 10:30         │
│ Mon  Meetings 3h  Focus 2h   │ 2. Thu no deep work           │
│ Tue  Meetings 4h  Focus 0h   │ 3. Fri overloaded             │
│ Wed  Conflict High           │                               │
│ Thu  No deep work            │ Suggestions                   │
│ Fri  Overloaded              │ 1. Move Interview             │
│                              │ 2. Protect Tue 13:30         │
└──────────────────────────────┴───────────────────────────────┘
```

### 验收标准

- 无日历源时显示空状态和导入入口。
- 有日历源时显示分析结果。
- 指标卡点击可跳转到对应详情页。
- 切换分析范围后重新分析。
- 刷新远程日历后结果更新。

---

## 9.3 Sources 页面

### 页面目标

管理日历源。

### 功能

- 列表展示所有日历源。
- 显示启用状态。
- 显示同步状态。
- 显示最近同步时间。
- 添加本地 ICS。
- 添加远程 ICS URL。
- 删除日历源。

### 线框

```text
┌──────────────────────────────────────────────────────────────┐
│ Calendar Sources                         [+ Add Source]       │
├──────────────────────────────────────────────────────────────┤
│ ● Work Calendar       Remote ICS     Enabled   Synced 10:32   │
│ ● Personal Calendar   Local File      Enabled   Imported       │
│ ○ Holidays            Remote ICS     Disabled  Failed          │
└──────────────────────────────────────────────────────────────┘
```

---

## 9.4 Conflicts 页面

### 页面目标

让用户查看、筛选、解释冲突。

### 功能

- 按时间排序。
- 按严重级别过滤。
- 按日历源过滤。
- 查看冲突详情。
- 忽略冲突。
- 复制建议。

### 线框

```text
┌──────────────────────────────────────────────────────────────┐
│ Conflicts                                                    │
├───────────────┬──────────────────────────────────────────────┤
│ Filters       │ [High] Wed 10:30-11:00                       │
│ Severity      │ Product Review overlaps Candidate Interview  │
│ Calendar      │ Overlap: 30m                                 │
│ Range         │ Suggestion: Move one event                   │
│               │ [Ignore] [Copy Suggestion]                   │
└───────────────┴──────────────────────────────────────────────┘
```

---

## 9.5 Free Time 页面

### 页面目标

展示用户未来一段时间内可用于专注工作的空闲块。

### 功能

- 按日期展示空闲块。
- 标记 Deep Work / Short Gap / Micro Gap。
- 支持最小专注块设置。
- 支持复制空闲时间。

### 线框

```text
┌──────────────────────────────────────────────────────────────┐
│ Free Time                         Minimum Deep Work: 90m      │
├──────────────────────────────────────────────────────────────┤
│ Monday                                                       │
│ 09:00-10:00  Short Gap                                       │
│ 13:30-15:30  Deep Work                                       │
│                                                              │
│ Tuesday                                                      │
│ No Deep Work Block                                           │
│ Best candidate: 15:00-16:00 Short Gap                        │
└──────────────────────────────────────────────────────────────┘
```

---

## 9.6 Focus 页面

### 页面目标

展示专注时间趋势和碎片化程度。

### 内容

- 本周 Deep Work 总时长。
- 每天最长空闲块。
- 无专注时间的日期。
- Fragmentation Score。
- Meeting Density。

### MVP 可视化方式

MVP 不强依赖图表库，可以用 CSS 和简单条形图完成。

后续 P1 可接入图表库。

---

## 9.7 Reports 页面

### 页面目标

让用户导出 Markdown / JSON 报告。

### 功能

- 选择报告范围。
- 选择导出格式。
- 选择隐私选项。
- 预览报告摘要。
- 保存到本地。
- 复制到剪贴板。

---

## 9.8 Settings 页面

### 设置项

| 分类 | 设置 |
|---|---|
| General | 语言、时区、一周开始日、日期格式 |
| Working Hours | 工作日、开始时间、结束时间、午休时间 |
| Analysis | 最小专注块、过载阈值、冲突严重度阈值 |
| Privacy | 隐私导出、清空本地缓存、事件标题遮蔽 |
| Advanced | 最大 ICS 文件大小、最大分析天数、最大重复实例 |

---

## 10. 前端技术方案：Next.js + React

## 10.1 技术栈

| 类型 | 技术 |
|---|---|
| 框架 | Next.js |
| UI 库 | React |
| 语言 | TypeScript |
| 样式 | Tailwind CSS |
| 组件基础 | Radix UI 或自定义组件 |
| 状态管理 | Zustand 或 React Context |
| 异步数据 | TanStack Query，可选 |
| 表单 | React Hook Form，可选 |
| 测试 | Vitest + React Testing Library |
| 桌面 API | `@tauri-apps/api` |

### 明确不使用

- Vue。
- Nuxt。
- Electron。
- Next.js API Routes。
- Next.js SSR 作为运行时依赖。

---

## 10.2 Next.js 配置

`next.config.mjs`：

```js
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
```

原因：

- Tauri 桌面应用加载的是静态前端资源。
- Next.js 静态导出会生成 `out` 目录。
- 默认 Next.js 图片优化依赖服务端能力，MVP 不使用。

---

## 10.3 Tauri 配置

`src-tauri/tauri.conf.json`：

```json
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:3000",
    "frontendDist": "../out"
  },
  "productName": "CalGuard",
  "version": "0.1.0",
  "identifier": "app.calguard.desktop"
}
```

---

## 10.4 前端调用 Rust 的方式

前端通过 Tauri command 调用 Rust：

```ts
import { invoke } from '@tauri-apps/api/core';

const report = await invoke<AnalysisReportDto>('analyze_calendar', {
  input: {
    rangeDays: 14,
    sourceIds: ['source_1'],
  },
});
```

### 约束

- 前端不直接读文件系统。
- 前端不直接访问 SQLite。
- 前端不直接下载远程 ICS。
- 前端只调用 Rust command。
- 所有核心计算都在 Rust。

---

## 10.5 前端目录结构

```text
apps/desktop/
├── app/
│   ├── page.tsx
│   ├── onboarding/page.tsx
│   ├── dashboard/page.tsx
│   ├── sources/page.tsx
│   ├── conflicts/page.tsx
│   ├── free-time/page.tsx
│   ├── focus/page.tsx
│   ├── reports/page.tsx
│   └── settings/page.tsx
├── components/
│   ├── layout/
│   ├── dashboard/
│   ├── sources/
│   ├── conflicts/
│   ├── free-time/
│   └── common/
├── lib/
│   ├── tauri.ts
│   ├── format-date.ts
│   ├── report.ts
│   └── types.ts
├── stores/
│   ├── app-store.ts
│   └── settings-store.ts
├── styles/
│   └── globals.css
└── src-tauri/
```

---

## 10.6 前端状态模型

### UI State

由 Zustand 或 React Context 管理：

```ts
type AppUiState = {
  selectedRangeDays: 7 | 14 | 30;
  selectedSourceIds: string[];
  conflictSeverityFilter: ConflictSeverity[];
  privacyMode: boolean;
};
```

### Server-like State

来自 Rust command 的数据可使用 TanStack Query 管理：

```text
calendarSources
analysisReport
settings
syncStatus
```

### 持久化边界

- 前端 store 只保存 UI 状态。
- 用户设置、日历源、缓存数据由 Rust + SQLite 持久化。


---

## 10.7 Tauri 插件与权限边界

MVP 只在前端使用 Tauri Dialog 插件做文件选择和保存路径选择，不在前端直接读取文件内容。

### 插件使用原则

| 能力 | 前端是否直接做 | 实现方式 |
|---|---:|---|
| 选择本地 `.ics` 文件 | 是 | `@tauri-apps/plugin-dialog` 的 open dialog |
| 读取 `.ics` 内容 | 否 | 将路径传给 Rust command，由 Rust 读取和校验 |
| 保存导出报告路径 | 是 | `@tauri-apps/plugin-dialog` 的 save dialog |
| 写入报告文件 | 否 | Rust command 写入 Markdown / JSON |
| 下载远程 ICS | 否 | Rust 使用 `reqwest` 下载 |
| 访问 SQLite | 否 | Rust 访问 SQLite |

### 推荐最小权限

`src-tauri/capabilities/default.json` 可按实际命令进一步收紧。MVP 至少需要：

```json
{
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "dialog:allow-message"
  ]
}
```

### 权限约束

- 不给前端开放通用文件系统读取权限。
- 不给前端开放任意 HTTP 请求权限。
- 不让前端直接接触 SQLite。
- Rust command 必须校验文件扩展名、文件大小和 URL 协议。
- 后续如引入 Tauri FS 或 HTTP 插件，必须重新做权限审查。

---

## 11. Rust 技术方案

## 11.1 Rust Workspace 结构

```text
calguard/
├── Cargo.toml
├── apps/
│   └── desktop/
│       ├── app/
│       ├── components/
│       ├── package.json
│       ├── next.config.mjs
│       └── src-tauri/
├── crates/
│   ├── calguard-core/
│   │   ├── src/model.rs
│   │   ├── src/parser.rs
│   │   ├── src/recurrence.rs
│   │   ├── src/interval.rs
│   │   ├── src/analysis.rs
│   │   ├── src/scoring.rs
│   │   └── src/report.rs
│   ├── calguard-store/
│   │   ├── src/db.rs
│   │   ├── src/migrations.rs
│   │   └── src/repository.rs
│   ├── calguard-sync/
│   │   ├── src/local_ics.rs
│   │   └── src/remote_ics.rs
│   └── calguard-tauri/
│       ├── src/commands.rs
│       ├── src/dto.rs
│       └── src/error.rs
├── fixtures/
│   ├── simple.ics
│   ├── conflicts.ics
│   ├── recurring-weekly.ics
│   ├── recurring-exdate.ics
│   ├── all-day.ics
│   ├── transparent.ics
│   └── cancelled.ics
└── docs/
    └── PRD.md
```

---

## 11.2 Rust crate 选型

| 功能 | crate | 用途 |
|---|---|---|
| 错误处理 | `thiserror` | 定义业务错误 |
| 临时错误聚合 | `anyhow` | 应用层快速传播错误 |
| 序列化 | `serde`, `serde_json` | DTO、报告、设置 |
| 时间处理 | `chrono`, `chrono-tz` | 日期、时区、时间计算 |
| ICS 解析 | `icalendar` 或 `ical` | 解析 iCalendar 文件 |
| RRULE | `rrule` | 重复事件展开 |
| SQLite | `rusqlite` | 本地存储 |
| HTTP | `reqwest` | 拉取远程 ICS |
| async runtime | `tokio` | 异步同步任务 |
| 日志 | `tracing` | 日志与调试 |
| 测试快照 | `insta` | 报告快照测试 |
| UUID | `uuid` | ID 生成 |

### 解析库注意事项

MVP 可以先选用 `icalendar` 或 `ical`，但必须用 fixture 验证真实 `.ics` 文件兼容性。若解析库无法覆盖常见场景，保留替换能力，解析模块不要把第三方 crate 类型泄漏到核心模型中。

---

## 11.3 Rust 分层职责

### calguard-core

负责纯业务逻辑：

- 事件模型。
- ICS 解析后的标准化模型。
- 重复事件展开。
- 区间合并。
- 冲突检测。
- 空闲时间计算。
- 健康评分。
- 报告生成。

要求：

- 不依赖 Tauri。
- 不依赖前端。
- 不直接访问 SQLite。
- 可被 CLI 或测试独立调用。

### calguard-store

负责本地持久化：

- SQLite 初始化。
- migration。
- CalendarSource 存储。
- Raw ICS 缓存。
- Event 存储。
- Settings 存储。
- Ignored items 存储。

### calguard-sync

负责日历同步：

- 读取本地 ICS 文件。
- 拉取远程 ICS URL。
- 限制文件大小。
- 处理超时。
- 返回原始 ICS 文本。

### calguard-tauri

负责 Tauri command：

- DTO 转换。
- 前后端边界。
- 错误转换。
- 调用 core/store/sync。

---

## 12. 数据模型

## 12.1 CalendarSource

```rust
pub struct CalendarSource {
    pub id: CalendarSourceId,
    pub name: String,
    pub kind: CalendarSourceKind,
    pub color: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
}

pub enum CalendarSourceKind {
    LocalIcsFile { path: PathBuf },
    RemoteIcsUrl { url: String },
}
```

---

## 12.2 Event

```rust
pub struct Event {
    pub id: EventId,
    pub source_id: CalendarSourceId,
    pub uid: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: DateTime<Tz>,
    pub ends_at: DateTime<Tz>,
    pub is_all_day: bool,
    pub status: EventStatus,
    pub transparency: Transparency,
    pub recurrence_rule: Option<String>,
    pub excluded_dates: Vec<DateTime<Tz>>,
    pub recurrence_id: Option<DateTime<Tz>>,
}
```

---

## 12.3 EventInstance

```rust
pub struct EventInstance {
    pub id: EventInstanceId,
    pub event_id: EventId,
    pub source_id: CalendarSourceId,
    pub uid: String,
    pub title: String,
    pub starts_at: DateTime<Tz>,
    pub ends_at: DateTime<Tz>,
    pub is_all_day: bool,
    pub status: EventStatus,
    pub transparency: Transparency,
}
```

---

## 12.4 Conflict

```rust
pub struct Conflict {
    pub id: ConflictId,
    pub event_ids: Vec<EventInstanceId>,
    pub starts_at: DateTime<Tz>,
    pub ends_at: DateTime<Tz>,
    pub overlap_minutes: i64,
    pub severity: ConflictSeverity,
    pub reason: String,
    pub ignored: bool,
}
```

---

## 12.5 FreeBlock

```rust
pub struct FreeBlock {
    pub id: FreeBlockId,
    pub starts_at: DateTime<Tz>,
    pub ends_at: DateTime<Tz>,
    pub duration_minutes: i64,
    pub block_type: FreeBlockType,
}
```

---

## 12.6 AnalysisReport

```rust
pub struct AnalysisReport {
    pub schema_version: String,
    pub period_start: DateTime<Tz>,
    pub period_end: DateTime<Tz>,
    pub generated_at: DateTime<Utc>,
    pub score: HealthScore,
    pub conflicts: Vec<Conflict>,
    pub free_blocks: Vec<FreeBlock>,
    pub overloaded_days: Vec<OverloadedDay>,
    pub focus_metrics: FocusMetrics,
    pub suggestions: Vec<Suggestion>,
}
```

---

## 13. SQLite 数据库设计

## 13.1 数据库原则

- SQLite 只存在本地。
- 默认不加密，P1 可支持可选加密。
- 远程 ICS 缓存只保存用户显式添加的 URL 内容。
- 用户可以清空缓存。
- 敏感字段导出前需要用户确认。

---

## 13.2 表结构

```sql
CREATE TABLE calendar_sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    config_json TEXT NOT NULL,
    color TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    sync_status TEXT NOT NULL DEFAULT 'idle',
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_synced_at TEXT
);

CREATE TABLE raw_calendars (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    raw_ics TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    etag TEXT,
    last_modified TEXT,
    imported_at TEXT NOT NULL,
    FOREIGN KEY(source_id) REFERENCES calendar_sources(id) ON DELETE CASCADE
);

CREATE TABLE events (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    uid TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    location TEXT,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    timezone TEXT,
    is_all_day INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    transparency TEXT NOT NULL,
    recurrence_rule TEXT,
    excluded_dates_json TEXT,
    recurrence_id TEXT,
    raw_calendar_id TEXT,
    FOREIGN KEY(source_id) REFERENCES calendar_sources(id) ON DELETE CASCADE,
    FOREIGN KEY(raw_calendar_id) REFERENCES raw_calendars(id) ON DELETE SET NULL
);

CREATE TABLE ignored_items (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    target_hash TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE report_exports (
    id TEXT PRIMARY KEY,
    format TEXT NOT NULL,
    file_path TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    privacy_mode INTEGER NOT NULL,
    created_at TEXT NOT NULL
);
```

---

## 14. Tauri Command API

## 14.1 命令列表

```rust
#[tauri::command]
async fn add_local_ics_source(input: AddLocalIcsSourceInput) -> Result<CalendarSourceDto, AppError>;

#[tauri::command]
async fn add_remote_ics_source(input: AddRemoteIcsSourceInput) -> Result<CalendarSourceDto, AppError>;

#[tauri::command]
async fn list_calendar_sources() -> Result<Vec<CalendarSourceDto>, AppError>;

#[tauri::command]
async fn update_calendar_source(input: UpdateCalendarSourceInput) -> Result<CalendarSourceDto, AppError>;

#[tauri::command]
async fn remove_calendar_source(source_id: String) -> Result<(), AppError>;

#[tauri::command]
async fn sync_calendar_source(source_id: String) -> Result<SyncResultDto, AppError>;

#[tauri::command]
async fn analyze_calendar(input: AnalyzeCalendarInput) -> Result<AnalysisReportDto, AppError>;

#[tauri::command]
async fn get_settings() -> Result<SettingsDto, AppError>;

#[tauri::command]
async fn update_settings(input: UpdateSettingsInput) -> Result<SettingsDto, AppError>;

#[tauri::command]
async fn export_report(input: ExportReportInput) -> Result<ExportReportResultDto, AppError>;

#[tauri::command]
async fn ignore_item(input: IgnoreItemInput) -> Result<(), AppError>;
```

---

## 14.2 DTO 示例

```rust
#[derive(Serialize, Deserialize)]
pub struct AnalyzeCalendarInput {
    pub range_days: u32,
    pub source_ids: Vec<String>,
    pub timezone: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisReportDto {
    pub schema_version: String,
    pub period_start: String,
    pub period_end: String,
    pub generated_at: String,
    pub score: HealthScoreDto,
    pub conflicts: Vec<ConflictDto>,
    pub free_blocks: Vec<FreeBlockDto>,
    pub overloaded_days: Vec<OverloadedDayDto>,
    pub focus_metrics: FocusMetricsDto,
    pub suggestions: Vec<SuggestionDto>,
}
```

---

## 14.3 错误返回格式

```rust
#[derive(Serialize, Deserialize)]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}
```

示例：

```json
{
  "code": "ICS_PARSE_FAILED",
  "message": "无法解析该日历文件，请确认它是有效的 .ics 文件。",
  "details": "line 42: invalid DTSTART",
  "recoverable": true
}
```

---

## 15. 核心算法

## 15.1 冲突检测算法

### 输入

```text
EventInstance[]
```

### 步骤

```text
1. 过滤 disabled source。
2. 过滤 CANCELLED event。
3. 过滤 TRANSPARENT event。
4. 过滤已忽略 event/conflict。
5. 按 starts_at 排序。
6. 遍历事件，检测时间区间重叠。
7. 合并同一时间段的多事件冲突。
8. 计算 severity。
```

### 伪代码

```rust
fn overlaps(a: &EventInstance, b: &EventInstance) -> bool {
    a.starts_at < b.ends_at && b.starts_at < a.ends_at
}

fn detect_conflicts(events: &[EventInstance]) -> Vec<Conflict> {
    let mut busy = events
        .iter()
        .filter(|event| event.is_busy())
        .collect::<Vec<_>>();

    busy.sort_by_key(|event| event.starts_at);

    let mut conflicts = Vec::new();

    for i in 0..busy.len() {
        for j in (i + 1)..busy.len() {
            if busy[j].starts_at >= busy[i].ends_at {
                break;
            }

            if overlaps(busy[i], busy[j]) {
                conflicts.push(build_conflict(busy[i], busy[j]));
            }
        }
    }

    conflicts
}
```

### MVP 性能说明

个人日历在 30 天窗口内事件数量通常不大，简单排序 + 局部遍历足够。若后续支持团队日历，可升级为 sweep line。

---

## 15.2 空闲时间算法

### 输入

```text
WorkingHourWindow[]
BusyInterval[]
UserSettings
```

### 步骤

```text
1. 根据用户设置生成每天工作时间窗口。
2. 收集当天 busy intervals。
3. 过滤透明、取消、禁用日历事件。
4. 合并重叠 busy intervals。
5. 用 working window 减去 busy intervals。
6. 得到 free blocks。
7. 根据时长分类。
```

### 伪代码

```rust
fn calculate_free_blocks(
    working_window: Interval,
    busy_intervals: Vec<Interval>,
    settings: AnalysisSettings,
) -> Vec<FreeBlock> {
    let merged_busy = merge_intervals(busy_intervals);
    let free_intervals = subtract_intervals(working_window, merged_busy);

    free_intervals
        .into_iter()
        .map(|interval| classify_free_block(interval, settings))
        .collect()
}
```

---

## 15.3 健康评分算法

### 输入

```text
conflicts
free_blocks
overloaded_days
focus_metrics
settings
```

### 输出

```text
HealthScore {
  score,
  grade,
  positive_reasons,
  negative_reasons
}
```

### 要求

- 每次加减分都要生成 reason。
- reason 要能定位到日期或事件。
- 算法不可依赖 UI 状态。
- 算法配置可以从 settings 读取。

---

## 16. 错误处理

## 16.1 错误分类

| code | 场景 | 用户提示 |
|---|---|---|
| FILE_NOT_FOUND | 本地 ICS 文件不存在 | 找不到该文件，请检查路径是否正确。 |
| FILE_TOO_LARGE | 文件超过限制 | 该日历文件过大，请换一个较小的分析范围或文件。 |
| ICS_PARSE_FAILED | ICS 解析失败 | 无法解析该日历文件。 |
| REMOTE_FETCH_FAILED | 远程 URL 拉取失败 | 无法下载远程日历，请检查网络或 URL。 |
| REMOTE_TIMEOUT | 远程请求超时 | 远程日历响应超时，请稍后重试。 |
| INVALID_URL | URL 非法 | 请输入有效的 HTTP/HTTPS URL。 |
| DB_ERROR | SQLite 错误 | 本地数据库访问失败。 |
| UNSUPPORTED_TIMEZONE | 时区不支持 | 当前时区无法识别，请在设置中选择其他时区。 |

## 16.2 错误展示原则

- 用户看到简洁可操作提示。
- 开发调试日志保留详细信息。
- 任何错误都不能导致 Rust panic。
- 单个日历源失败不影响其他日历源。

---

## 17. 隐私与安全

## 17.1 隐私原则

```text
默认本地处理。
默认不注册账号。
默认不上传日历。
默认不写回日历。
默认不包含事件描述导出。
允许用户清空本地缓存。
允许隐私模式导出。
```

## 17.2 远程 ICS 安全限制

| 限制 | MVP 默认值 |
|---|---:|
| 请求超时 | 10 秒 |
| 单个 ICS 最大大小 | 20 MB |
| 单个日历源最大事件数 | 50,000 |
| 单次分析窗口最大 | 30 天，设置中最高 180 天 |
| 单个重复事件最大展开实例 | 1000 |
| URL 协议 | HTTP/HTTPS，优先 HTTPS |

## 17.3 不存储内容

MVP 不存储：

- Google OAuth token。
- Outlook OAuth token。
- CalDAV 密码。
- 云端分析副本。
- 用户身份账户。

原因：MVP 不做这些能力。

---

## 18. 性能要求

| 场景 | MVP 目标 |
|---|---:|
| 应用冷启动 | < 3 秒 |
| 导入 1000 个事件 | < 3 秒 |
| 分析 30 天 | < 1 秒 |
| Dashboard 切换范围 | < 500 ms |
| 远程 ICS 请求超时 | 10 秒 |
| 数据库大小 | 普通个人使用 < 100 MB |
| UI 交互响应 | < 100 ms |

---

## 19. 测试计划

## 19.1 Rust 单元测试

必须覆盖：

- `overlaps()`。
- `merge_intervals()`。
- `subtract_intervals()`。
- `detect_conflicts()`。
- `calculate_free_blocks()`。
- `score_health()`。
- `expand_recurring_events()`。

## 19.2 Fixture 测试

至少准备：

| 文件 | 覆盖内容 |
|---|---|
| simple.ics | 普通事件 |
| conflicts.ics | 冲突事件 |
| recurring-weekly.ics | 每周重复 |
| recurring-daily.ics | 每日重复 |
| recurring-exdate.ics | EXDATE 排除 |
| all-day.ics | 全天事件 |
| transparent.ics | TRANSPARENT |
| cancelled.ics | CANCELLED |
| timezone.ics | 时区事件 |
| no-dtend-duration.ics | DURATION |
| malformed.ics | 解析失败 |

## 19.3 前端测试

必须覆盖：

- Dashboard 空状态。
- Dashboard 有数据状态。
- Sources 添加表单校验。
- Conflicts 过滤。
- Free Time 展示。
- Settings 表单。
- Reports 隐私模式切换。

## 19.4 端到端手工测试

MVP 发布前必须完成：

1. macOS 导入本地 ICS。
2. Windows 导入本地 ICS。
3. Linux 导入本地 ICS。
4. 远程 ICS URL 成功拉取。
5. 远程 ICS URL 失败时应用不崩溃。
6. 删除日历源后结果更新。
7. 清空缓存后应用可重新导入。
8. Markdown 报告可打开。
9. JSON 报告 schema 正确。

---

## 20. 开发计划

## 20.1 推荐开发顺序

```text
1. Rust core models
2. Interval algorithms
3. ICS parser + fixtures
4. Recurrence expansion
5. SQLite store
6. Tauri commands
7. Next.js + React UI skeleton
8. Dashboard
9. Sources / Conflicts / Free Time
10. Reports / Settings
11. Packaging and README
```

不要先做漂亮 UI。先确保 core 算法可靠。

---

## 20.2 8 周开发计划

### Week 1：Rust 核心模型与区间算法

交付物：

- `Event`
- `EventInstance`
- `Interval`
- `Conflict`
- `FreeBlock`
- `overlaps()`
- `merge_intervals()`
- `subtract_intervals()`

验收：

- 核心算法单元测试通过。

---

### Week 2：ICS 解析与 fixture

交付物：

- 本地 `.ics` 读取。
- 基础 `VEVENT` 解析。
- 20 个 fixture 初版。

验收：

- 能解析普通事件、全天事件、取消事件、透明事件。

---

### Week 3：重复事件与分析引擎

交付物：

- RRULE 展开。
- 冲突检测。
- 空闲时间计算。
- 健康评分。

验收：

- `fixtures/conflicts.ics` 能生成稳定分析报告。

---

### Week 4：SQLite 与 Tauri Commands

交付物：

- SQLite schema。
- migrations。
- sources CRUD。
- `analyze_calendar` command。
- `export_report` command。

验收：

- 前端可以通过 Tauri command 获得分析报告。

---

### Week 5：Next.js + React UI 骨架

交付物：

- Next.js static export 配置。
- Tauri 集成。
- Layout。
- Navigation。
- Onboarding。
- Sources 页面。

验收：

- `pnpm tauri dev` 能打开桌面窗口。
- 用户能添加本地 ICS。

---

### Week 6：Dashboard、Conflicts、Free Time

交付物：

- Dashboard 指标卡。
- 冲突列表。
- 空闲时间列表。
- 过滤器。

验收：

- 导入 fixture 后可看到完整分析结果。

---

### Week 7：Reports、Settings、隐私模式

交付物：

- Markdown 导出。
- JSON 导出。
- 隐私导出选项。
- Settings 页面。

验收：

- 导出文件可读。
- 设置变更会影响分析结果。

---

### Week 8：发布准备

交付物：

- README。
- 截图。
- 示例数据。
- GitHub Release。
- macOS / Windows / Linux 打包配置。
- Issue templates。

验收：

- 新用户根据 README 可安装并运行。

---

## 21. GitHub 仓库规划

## 21.1 README 标题

```md
# CalGuard

A local-first calendar health analyzer built with Rust, Tauri, Next.js and React.
```

## 21.2 README 核心特性

```md
- Local-first calendar analysis
- Built with Rust, Tauri, Next.js and React
- Import local `.ics` files
- Subscribe to remote ICS URLs
- Detect meeting conflicts
- Find deep work blocks
- Identify overloaded days
- Generate calendar health scores
- Export Markdown and JSON reports
- Privacy-first report mode
```

## 21.3 README 非目标

```md
CalGuard is not a full calendar client.
It does not modify your calendar.
It does not upload your events to a cloud service.
It does not require an account.
```

---

## 22. 风险与应对

## 22.1 ICS 标准复杂

风险：真实世界 `.ics` 文件格式差异大，RRULE、时区、例外日期复杂。

应对：

- MVP 支持常见 80% 场景。
- 建立 fixture 测试。
- 解析失败不影响整个日历。
- 保留 raw ICS 方便后续修复。

---

## 22.2 Next.js 与 Tauri 架构误用

风险：误用 Next.js 服务端能力，导致 Tauri 打包后不可用。

应对：

- 强制 `output: 'export'`。
- 禁止 API Routes、SSR、Server Actions 作为 MVP 依赖。
- 所有系统能力走 Tauri command。
- 打包测试作为 PR 必过项。

---

## 22.3 GUI 开发拖慢 Rust 学习

风险：过早追求 UI 细节，导致 Rust core 不稳定。

应对：

- 先写 `calguard-core`。
- UI 只消费 DTO。
- 不在前端实现核心算法。
- MVP 图表简单化。

---

## 22.4 产品范围膨胀

风险：想同时做日历、任务、AI、预约链接，导致 MVP 失败。

应对：

- MVP 坚持只读诊断。
- 不做创建/编辑/删除事件。
- 不做写回。
- 不做账号系统。

---

## 22.5 隐私信任不足

风险：用户担心日历数据泄露。

应对：

- README 强调本地优先。
- 应用内展示隐私说明。
- 导出前提示敏感信息。
- 提供隐私模式导出。
- 提供清空缓存。

---

## 23. 成功指标

## 23.1 产品指标

| 指标 | MVP 目标 |
|---|---:|
| 首次导入成功率 | > 80% |
| 首次看到报告时间 | < 3 分钟 |
| 用户添加多个日历源比例 | > 30% |
| 报告导出使用率 | > 10% |
| 用户一周后再次打开比例 | > 20% |

## 23.2 技术指标

| 指标 | MVP 目标 |
|---|---:|
| Rust core 测试覆盖率 | > 70% |
| fixture 文件数量 | >= 20 |
| panic 数量 | 0 |
| 分析 1000 个事件耗时 | < 1 秒 |
| 跨平台打包 | macOS / Windows / Linux |

## 23.3 开源指标

| 指标 | MVP 发布后目标 |
|---|---:|
| GitHub Stars | 100+ |
| Issues / Discussions | 20+ |
| 外部贡献者 | 3+ |
| README 安装成功率 | 高 |
| 示例截图完整度 | 高 |

---

## 24. MVP 验收清单

## 24.1 产品验收

- [ ] 用户无需注册即可使用。
- [ ] 用户可导入本地 `.ics` 文件。
- [ ] 用户可添加远程 `.ics` URL。
- [ ] 用户可设置工作时间。
- [ ] 用户可选择 7 / 14 / 30 天分析范围。
- [ ] 用户可看到 Dashboard。
- [ ] 用户可看到冲突列表。
- [ ] 用户可看到空闲时间列表。
- [ ] 用户可看到健康分数。
- [ ] 用户可看到扣分原因。
- [ ] 用户可导出 Markdown。
- [ ] 用户可导出 JSON。
- [ ] 用户可启用隐私导出。
- [ ] 用户可清空本地缓存。

## 24.2 技术验收

- [ ] Next.js 使用 `output: 'export'`。
- [ ] Tauri `frontendDist` 指向 `../out`。
- [ ] 不使用 Vue。
- [ ] 不依赖 API Routes。
- [ ] 不依赖 SSR。
- [ ] 不依赖 Server Actions。
- [ ] 核心分析逻辑在 Rust。
- [ ] SQLite 数据只保存在本地。
- [ ] 单元测试通过。
- [ ] fixture 测试通过。
- [ ] 打包后应用可运行。

## 24.3 安全与隐私验收

- [ ] 默认不上传日历。
- [ ] 默认不写回日历。
- [ ] 远程 ICS 请求有超时。
- [ ] 单个 ICS 文件有大小限制。
- [ ] 导出前有隐私提醒。
- [ ] 隐私模式会隐藏事件标题和地点。
- [ ] 用户可清空缓存。

---

## 25. PRD 自检 Review

本节用于检查本文档是否存在前后矛盾、范围膨胀或与 MVP 不一致的问题。

### 25.1 技术一致性检查

| 检查项 | 结论 |
|---|---|
| 是否明确使用 Next.js + React | 通过 |
| 是否移除 Vue 作为方案 | 通过 |
| 是否误用 Next.js 服务端能力 | 未作为 MVP 依赖，已禁止 |
| 是否明确 Tauri 静态导出约束 | 通过 |
| 是否将核心计算放在 Rust | 通过 |
| 是否避免前端直接访问 SQLite | 通过 |
| 是否避免前端直接读文件系统 | 通过 |

### 25.2 产品范围检查

| 检查项 | 结论 |
|---|---|
| 是否仍然是 MVP | 通过 |
| 是否避免完整日历 App 范围 | 通过 |
| 是否避免日历写回 | 通过 |
| 是否避免 OAuth 和 CalDAV | 通过，均未进入 P0，统一作为后续版本 |
| 是否避免 AI 自动排期 | 通过 |
| 是否保留真实用户价值 | 通过，冲突、空闲、健康报告形成闭环 |

### 25.3 功能闭环检查

| 用户动作 | 是否闭环 |
|---|---|
| 导入 ICS | 是 |
| 远程 URL 同步 | 是 |
| 查看健康概览 | 是 |
| 查看冲突原因 | 是 |
| 查看专注时间 | 是 |
| 修改设置后重新分析 | 是 |
| 导出报告 | 是 |

### 25.4 风险检查

| 风险 | 是否有应对 |
|---|---|
| ICS 复杂 | 有 fixture 和范围限制 |
| Next.js 打包失败 | 有 static export 约束 |
| 范围膨胀 | P0/P1/P2 已拆分 |
| 隐私担忧 | 本地优先、隐私导出、清空缓存 |
| GUI 分散 Rust 学习 | 先 core 后 UI 的开发顺序 |

### 25.5 最终 Review 结论

本文档已将产品范围收敛为一个可完成的 MVP：

```text
本地/远程 ICS 导入
→ Rust 本地分析
→ Next.js + React GUI 展示
→ 冲突、空闲时间、健康分数
→ Markdown / JSON 导出
```

不存在以下前后矛盾：

- 一边要求 Tauri 静态前端，一边依赖 Next.js 服务端能力。
- 一边要求 MVP，一边把 CalDAV/OAuth/AI/写回放入 P0。
- 一边要求本地优先，一边要求云端账号或上传分析。
- 一边要求学习 Rust，一边把核心算法放到前端。
- 一边要求 Next.js + React，一边引入 Vue。

---

## 26. 参考资料

1. Tauri 官方 Next.js 指南：<https://v2.tauri.app/start/frontend/nextjs/>
2. Tauri 官方创建项目指南：<https://v2.tauri.app/start/create-project/>
3. Tauri 官方 “Calling Rust from the Frontend”：<https://v2.tauri.app/develop/calling-rust/>
4. Tauri 官方 Dialog 插件文档：<https://v2.tauri.app/plugin/dialog/>
5. Tauri 官方 Permissions 文档：<https://v2.tauri.app/security/permissions/>
6. Next.js 官方 Static Export 文档：<https://nextjs.org/docs/pages/guides/static-exports>
7. React 官方 Hooks 文档：<https://react.dev/reference/react/hooks>
8. iCalendar RFC 5545：<https://datatracker.ietf.org/doc/html/rfc5545>
9. `rusqlite` 文档：<https://docs.rs/rusqlite/>
10. `rrule` 文档：<https://docs.rs/rrule>
11. `icalendar` 文档：<https://docs.rs/icalendar/>

---

## 27. 附录：MVP 首屏文案草案

```text
CalGuard

Find what is wrong with your calendar.

CalGuard analyzes your local or remote ICS calendars to detect conflicts,
missing focus time, overloaded days, and fragmented schedules.

Your calendar data stays on your device by default.

[Import .ics File] [Add ICS URL]
```

中文版本：

```text
CalGuard

发现你的日程问题。

CalGuard 可以分析本地或远程 ICS 日历，检测日程冲突、专注时间不足、会议过载和时间碎片化。

默认情况下，你的日历数据只保存在本机。

[导入 .ics 文件] [添加 ICS URL]
```

