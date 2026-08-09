# Agent 导向 CLI UX 业界 SOTA 对照调研报告

- 调研日期：2026-08-09（归档编号沿用任务基准日 2026-08-08）
- 任务：Task ID 4（CLI UX 完整重设计的外部基准调研，纯调研，不修改任何源码）
- 方法：公开网络检索 + 一手来源全文阅读；每条结论附来源链接，并标注【可直接采纳】/【仅供参考】
- 前置阅读：docs/researches/cli-ux-agent-visible-output-research-2026-08-08.md（v0.4 现状调研）
- 待评估的重设计核心命题：把目标文件路径与使用者名字提升为前置必填位置参数，形如 paperwork <thread文件> <名字> send {参数}

---

## 1. 方向一：agent 导向的 CLI/工具接口设计实践

### 1.1 业界已有系统评估 rubric：agent-ready CLI 七原则
Trevin Chow 的文章是当前最系统的 agent CLI 设计 rubric，每条原则按 Blocker（agent 无法使用）/ Friction（可用但低效）/ Optimization 三级定级。
来源：https://trevinsays.com/p/7-principles-for-agent-friendly-clis
1) 默认非交互：stdin 非 TTY 时绝不能弹提示，否则挂起 = Blocker；应支持 --no-input/--yes。
2) 结构化可解析输出：数据命令必须有 --json；成功 0/失败非 0；结果走 stdout、诊断走 stderr；非 TTY 抑制 ANSI 颜色与装饰。
3) 快速失败且可执行的错误：错误要指明具体问题、给出正确调用形态、建议合法取值、附示例；"Error: --content is required" 远好于 "missing required arguments"。
4) 安全重试与显式变更边界：agent 会自动重试，变更命令须让重复可检测，成功输出返回标识符供后续引用；危险操作要显式 flag + --dry-run。
5) 渐进式帮助：agent 按 顶层 --help → 子命令 --help → 示例 的路径学习，两次调用内必须能拿到完整调用形态；示例比描述更重要。
6) 可组合与可预测：stdin/stdout 管道、跨子命令命名与结构一致（--limit 不能一处叫 --max-results）。
7) 有界高信号响应：默认窄输出；截断时必须教 agent 如何收窄查询（"Showing 25 of 312" + 收窄命令建议）。
对照评估：paperwork v0.4 已满足 2/3/5/7（envelope、fix:+example:、--limit 20、conclusion 首行）；1 与 4 经源码核验也满足（全命令无交互提示；send 回执 #N 即标识符）。【可直接采纳】作为重设计验收清单。

### 1.2 十原则更新版（2026-05）：从"不弄坏 agent"到"越用越强"
来源：https://trevinsays.com/p/10-principles-for-agent-native-clis
前五条压缩自七原则，新增五条：
6) 跨 CLI 词汇一致性（作者最确信的一条）：agent 不是逐个学 CLI，而是从见过的所有 CLI 建立泛化模型。用 get 而非 info、list 而非 ls、--force 而非 --skip-confirmations、--json 而非 --format=json；偏离惯例不会让 agent 失败，只会让它"缓慢成功"——多烧 --help 与重试。Cloudflare 在 schema 层强制执行这类规则。
7) 三层内省：--help（人读）/ agent-context 式的带版本机器可读 schema JSON / SKILL.md 工作流教学（HeyGen 随 CLI 发布 skills 仓库）。
8) 异步感知：--wait + 持久 job ledger（paperwork 为同步文件操作，不适用）。
9) 持久身份 profile：显式 flag > 环境变量 > profile > 默认值。注意：与 paperwork 的无状态哲学方向相反，见 5.2 讨论。
10) 双向 I/O：--deliver 把产物直接路由到 stdout/file/webhook；feedback 命令让 agent 上报摩擦。
对本项目：6) 词汇一致性 → 需要成文命名政策并用测试强制；7) 三层内省 → paperwork 目前只有 --help 一层，缺 SKILL.md 与机器可读 schema【可直接采纳，属新增项】；9) 不采纳（违背 ADR-011 无状态原则）。

### 1.3 Cloudflare：agent 是首要客户，schema 单源生成全部界面
来源：https://blog.cloudflare.com/cf-cli-local-explorer/（2026-04-13）
- 原文立场："Increasingly, agents are the primary customer of our APIs."
- 用一份 TypeScript schema 生成 CLI、SDK、Terraform provider、MCP server、文档，覆盖近 3000 个 API 操作；命名规则在 schema 层强制："always get, never info；always --force, never --skip-confirmations；always --json, never --format"。
- 关键判断："manually enforcing consistency through reviews is Swiss cheese"（靠评审人工维护一致性必然漏）。
- Code Mode MCP 用 <1000 tokens 服务全部 API —— 界面描述本身也有 token 预算。
对本项目：命名政策必须可机械检查。paperwork 已有 cli_integration.rs 对 envelope 的精确断言，可同款扩展为"动词表/flag 表白名单"测试。【可直接采纳】

### 1.4 Anthropic 官方工具设计原则（MCP 工具，对 CLI 同样适用）
来源：https://www.anthropic.com/engineering/writing-tools-for-agents
- 少而精的工具优于 1:1 包装 API：实现 search_contacts 而非 list_contacts；工具可合并多步操作、附带相关元数据。
- 命名空间化减少选择混淆（asana_search vs jira_search）；工具名反映任务的自然分割。
- 返回高信号上下文：自然语言名字显著优于 UUID 等密码式标识（减少幻觉）；提供 response_format=concise/detailed 两档（Slack 示例 206 tokens → 72 tokens）。
- 分页/过滤/截断要有合理默认；截断时引导 agent 用更窄的查询。
- 错误响应要 prompt-engineer：给出具体、可执行的改进建议，而非不透明的错误码或堆栈；截断提示同样用来引导更省 token 的策略。
- 参数命名必须无歧义：用 user_id 而非 user。
- 响应结构（XML/JSON/Markdown）无普适最优，要靠自己的 eval 实测。
对本项目：post send 的 --from（发送者）与 post read 的 --from/--to（序列号范围）是"参数名歧义"的典型反例，重设计必须拆开（现状调研第 7 章已列为未解决 ISSUE）。concise/detailed 两档与 paperwork 的 summary/read 分层同构，应保持。【可直接采纳】

### 1.5 Infracost 实测：为 agent 重设计 CLI 省 79% token（最重要的定量证据）
来源：https://www.infracost.io/resources/blog/we-cut-claude-s-token-usage-79-by-redesigning-our-cli-for-agents（2026-05-18）
- 基准：16 个问题 × 5 次重复 × 3 配置（bare Claude / skill+--llm / skill+--json），Opus 模型，1171 个资源的 Terraform fixture。
- 措施一：谓词下推。把 agent 原本用 jq/python 管道自行拼的过滤吸收进 CLI flag（--filter、--fields、-addresses-only）。效果：难题桶输出 tokens 113K→24K（-79%），成本 -67%，难题正确率 0/6→6/6。
- 措施二：TOON 格式（https://toonformat.dev/）。对均匀对象数组只写一行表头 + 逗号分隔值行，避免 JSON 重复字段名；比 compact JSON 再省 30-40% tokens，理解准确率持平（76.4% vs 75.0%）。
- 措施三：SKILL.md 教工作流。难题从 0/6 到 6/6 的关键不是输出格式，而是 skill 直接教"这类问题用这条命令"（--summary --fields 预算好的去重计数，跳过 wc -l 管道）。
- 方法论：没有基准 harness，这些缺口"不可见"；改进本身不新颖（谓词下推、源端投影、省脚手架的传输格式），新颖的是按 token 计价的消费者。
对本项目：
a) post read 的 --mention/--reply-to/--limit 即谓词下推，方向正确；重设计应继续把高频过滤吸收进 CLI，而非让 agent 直接 grep 文件。【可直接采纳】
b) summary 先于细读（post summary）正是"摘要+标识符先于原始细节"，保持。
c) 随仓库提供 SKILL.md（教 agent 每个原语的典型工作流）是低成本高回报动作。【可直接采纳】
d) TOON 不引入：收益阈值为数百行均匀数组（Infracost 场景是 200KB+ dump），paperwork 单线程 read 默认 20 条，达不到收益门槛，且生态成本高。【仅供参考：不采纳】

### 1.6 反方观点与争议（防止设计一边倒）
- HN "Principles for agent-native CLIs" 讨论（https://news.ycombinator.com/item?id=48052333）：高赞观点指出"小数据量时自然语言纯文本几乎总是优于 JSON"——LLM 训练语料以自然语言为主，JSON 字段名是冗余 token；只有大数据量才需要机器可读以便脚本化处理。"LLM 需要机器可读输出"被斥为常见误区。
- 另有评论反对"三层内省"中的机器可读 schema 层：把一次可懂的输出变成需多次推理的复杂 JSON 是浪费算力。
- 设计立场分歧：一方主张"为 agent 优先设计"（Trevin/Cloudflare）；另一方主张先为人和程序设计，agent 是"统计平均用户"，工具应匹配模型既有预期而非发明新文法（https://news.ycombinator.com/item?id=48054553）。
- 实证支持"CLI 是 agent 最顺手的界面"：CLI 是 LLM 训练数据中最稠密的工具使用模式（Manus 前后端负责人总结，https://www.reddit.com/r/LocalLLaMA/comments/1rrisqn/）；Karpathy 2026-02 "Build. For. Agents."（firecrawl 文转引）。
- 工程实践：在错误文本中直接嵌入面向 agent 的指导段（"GUIDANCE FOR THE AI AGENT"）以纠正其规避行为，已被实战采用（https://www.notcheckmark.com/2025/07/rethinking-cli-interfaces-for-ai/）。
对本项目（作为设计约束）：paperwork 的默认 envelope（自然语言式 + 固定行首语法 + --json 可选）恰好落在"自然语言 vs JSON"争议的最优中点，应保持；任何新文法（包括前置位置参数）都必须评估"与模型既有预期的距离"，并用 help 示例、SKILL.md、错误自愈三件套补偿偏离。【可直接采纳】

---

## 2. 方向二：成熟 CLI 的文法惯例

### 2.1 与本次重设计直接相关的两条经典规则
a) clig.dev（Command Line Interface Guidelines，https://clig.dev/）：UNIX 传统的现代修订版。与本任务相关的要点：成功退出码 0、把非零码映射到最重要的失败模式；诊断走 stderr；在不损害可用性的前提下提供机器可读输出；--json 输出结构化数据；--plain 输出纯表格文本供 grep/awk；-q 静默非必要输出；帮助文本"示例优先"；用户拼错时给出猜测建议（如 brew update jq → brew upgrade jq）。
b) jmmv "CLI design: Putting flags to good use"（https://jmmv.dev/2013/08/cli-design-putting-flags-to-good-use.html）：flag 即"选项"，语义上不允许必填；必填参数应做成位置参数或 key=value 命名参数。文中把 add_user --user=x --group=y（两个都必填）修正为 add_user user group，可选的 --shell/--uid 保留为 flag；动作选择必须用子命令而非 --set/--unset 式 flag。
=> 本次重设计"把文件路径 + 使用者名字提升为前置必填位置参数"正是该规则的直接应用：当前 --from 与 PATH 均为必填却占据 flag/混合位置。【可直接采纳】
c) 平衡观点（StackExchange "Positional arguments vs options"，https://softwareengineering.stackexchange.com/questions/366218/）：位置参数简洁但顺序是记忆负担、不自描述；flag 冗长但自描述、顺序无关。通行做法：只把 1-2 个"永远存在的核心操作数"设为位置参数，其余一律 flag。paperwork 的文件路径与（写操作的）使用者名字正是此类核心操作数。

### 2.2 成熟工具文法普查
| 工具 | 文法形态 | 核心操作数位置 | 身份机制 | 来源 |
|------|----------|----------------|----------|------|
| git | 扁平动词（git commit/push） | 动词后位置参数（git push <remote> <branch>） | config 默认身份 + --author 覆盖 | https://git-scm.com/docs/git-commit |
| kubectl | 动词+资源（kubectl get pods <name>） | 资源名在动词后作位置参数 | kubeconfig 上下文 + 全局 --as/--as-group 覆盖 | https://kubernetes.io/docs/reference/access-authn-authz/user-impersonation/ |
| docker | 名词+动词管理命令（docker container exec） | CONTAINER 必填位置参数，先于 COMMAND | -u/--user flag，置于容器名之前 | https://docs.docker.com/reference/cli/docker/container/exec/ |
| gh | 名词+动词（gh pr list / gh issue create） | 仓库经登录态隐式确定 | gh auth login（有状态） | https://cli.github.com/manual/ |
| ripgrep | 无子命令（rg PATTERN [PATH]） | 模式第一、路径可选在后 | 不适用 | https://github.com/BurntSushi/ripgrep |
| sudo / runuser | sudo [-u user] command / runuser -u user -- command | 被执行的命令在最后 | -u flag 置于动作之前 | https://man7.org/linux/man-pages/man8/sudo.8.html |
| ssh | ssh [user@]host [command] | destination 位置参数在最前 | 身份以 user@ 前缀并入 destination；另有 -l | https://man7.org/linux/man-pages/man1/ssh.1.html |
| psql / mysql | psql -U user dbname | 库名位置参数 | -U/-u flag | https://www.postgresql.org/docs/current/app-psql.html |

### 2.3 提炼出的四条规律
1) noun-verb vs verb-noun：git 式扁平动词适合动词集很小的工具；资源导向的现代 CLI（docker/gh，kubectl 的 API 语义亦是资源中心）一律先按名词分组再跟动词。paperwork 现状即 noun-verb（post send / brief verify），与主流一致。【可直接采纳：保持 noun-verb，不改回 verb-noun】
2) 核心操作数用位置参数且必填：文件路径/容器名/资源名在上表全部工具中都是必填位置参数。paperwork 的 PATH 已是位置参数；把使用者名字也提升为位置参数，最近似的先例是 ssh 的 user@ 融合式写法。
3) 名词 scope 内动词居尾是常态：gh pr list、docker image rm、docker container exec 都是"scope 名词...动词收尾"。paperwork <file> <name> send 读作"scope（文件）→ 限定（身份）→ 动词"，与该模式文法连续。
4) 必填参数前置：sudo/runuser/docker exec 都把身份或目标修饰放在动作对象之前，"上下文在前、动作收尾"是成熟模式；把文件与名字放在动词之前符合该模式。【可直接采纳】

---

## 3. 方向三：机器可读输出契约与错误自恢复

### 3.1 结构化输出契约的行业共识
- --json 是唯一标准 flag 名：Cloudflare "always --json, never --format=json"；Trevin 指出 flag 名不一致本身构成一类损坏。paperwork 已符合。【可直接采纳：保持，且不引入任何别名】
- 退出码分级：clig.dev 建议把非零码映射到最重要失败模式；Trevin 示例对 not-found 用专门退出码 4。paperwork 现统一 exit 1、以 category 词（validation/not-found/...）承载分类，--json 内含 category 与 exit_code 字段。【仅供参考：可选增强】把 category 映射为不同退出码（agent 不解析输出也能路由）；但现行 category 词已机器可读，收益有限，属可选项。
- 错误输出流：主流为 stderr（clig.dev/Trevin）。paperwork 默认模式符合；--json 模式错误走 stdout，属有意偏离（便于程序化捕获）。【仅供参考】保持可以，但应在 README/help 显著声明，避免与"诊断走 stderr"预期冲突。
- JSON envelope 形态：gh/kubectl 的 -o json 输出"纯数据对象"；paperwork 的 {"status","command","conclusion",...} 属"操作结果契约"，与 Anthropic "返回有意义的上下文"（status + 一句话结论供首行决策）一致。【可直接采纳：保持 status/conclusion 字段】

### 3.2 actionable errors：业界共识度最高的一点
- Trevin 原则 3 的细化："错误是 agent 收到的最高信号上下文"——它恰好在 agent 不知道下一步做什么时触发。好错误四要素：点名具体问题 / 给正确调用形态 / 枚举合法取值 / 给示例。范例：error: --visibility must be one of: public, private, unlisted (got: "secret")。
- Anthropic：错误响应要 prompt-engineer，给具体可执行的改进建议，而非不透明错误码/堆栈；输入校验失败时正是引导 agent 改参数的好时机。
- Elm "Compiler Errors for Humans"（https://elm-lang.org/news/compiler-errors-for-humans）与 Temporal "Write errors that don't make me think"（https://temporal.io/blog/error-message-design）：错误即教学，写给使用者而非维护者。
- agent 专属变体：在错误文本中直接嵌入 "GUIDANCE FOR THE AI AGENT" 指导段（notcheckmark 的 git 防绕过包装，见 1.6）。
- paperwork 的 error <category>: + fix: + example: 三层结构已实现四要素中的三项，属业界前列。
【可直接采纳的增强项】a) 补齐"枚举合法取值"：拒绝 actor 名字时列出线程 participants（或 contacts 名录中的名字）；枚举型参数非法时列候选集。这是前置 actor 设计下最高价值的落点。b) 缺参错误必须附"一条完整正确命令"示例。c) example 行保持永远可直接复制执行。

---

## 4. 方向四：MCP/agent 工具协议 vs CLI 形态

### 4.1 token 经济学对比（支持 CLI 押注）
- scalekit 基准（https://github.com/scalekit-inc/mcp-vs-cli-benchmark）：同一组 GitHub 任务，MCP 比 CLI 多耗 1.3x~80x tokens，主要来自每请求携带的工具 schema；CLI 在复杂任务上需要更多试错调用；两者任务完成率均 100%；MCP 在结构化数据任务上更快（直接 API vs 解析输出）。
- firecrawl 综述（https://www.firecrawl.dev/blog/mcp-vs-cli）：CLI 每命令约 200 tokens vs MCP 全量 schema 预载可达数万 tokens（GitHub MCP 约 44K）；Microsoft Graph 场景 CLI 方案便宜约 35x；Anthropic "写代码调工具"模式曾把单任务 150K tokens 降到 2K。MCP 的不可替代面：per-user OAuth、治理、超大工具集分发（Cloudflare Code Mode 用 schema 预算把 3000 操作压进 <1000 tokens，说明问题在界面描述失控而非协议本身）。
- Trevin：CLI 文本进文本出、零 schema 开销，LLM 从训练数据已熟悉常见 CLI；MCP 只在需要 per-user auth/治理时才值得其复杂度。

### 4.2 界面设计取向对照
| 维度 | MCP | CLI |
|------|-----|-----|
| 调用形态 | JSON schema 参数对象，字段自描述 | 位置+flag，依赖既有知识/help |
| 发现性 | tools/list 一次性全量 schema（token 贵） | --help 分层发现（token 廉、但多轮次） |
| 错误通道 | isError + 结构化 content | 退出码 + stderr/envelope |
| 身份 | 多绑定服务端会话/token | 每次调用显式传入（契合无状态） |
| 适配场景 | 有状态服务、多用户鉴权、治理 | 本地文件、无状态、无鉴权 |
结论：paperwork 的场景（本地文件、无状态、无鉴权、路径显式）恰是 CLI 相对 MCP 优势最大的区间；MCP 的两大优势（鉴权/发现性）在此不成立。【可直接采纳：战略层确认继续押注 CLI 形态，无需迁移 MCP】若未来提供 MCP，HN 中的成熟模式是"CLI 与 MCP 作为同一功能层的两个薄门面"（https://news.ycombinator.com/item?id=48052333）。

---

## 5. 方向五："身份/actor 作为前置位置参数"的先例普查

### 5.1 既有机制对照
| 先例 | 形态 | 性质 | 来源 |
|------|------|------|------|
| git commit --author=<a> | flag，可选覆盖 | 默认身份来自 config；身份缺失时的错误本身就是 actionable（直接给出 git config 命令） | https://git-scm.com/docs/git-commit |
| sudo -u <user> <cmd> / runuser -u <user> -- <cmd> | flag，置于动作之前 | 显式身份切换，身份先于动作 | https://man7.org/linux/man-pages/man8/sudo.8.html |
| kubectl --as=<user> --as-group=<g> | 全局 flag，逐命令携带 | impersonation 审计场景，无默认值时必须显式 | https://kubernetes.io/docs/reference/access-authn-authz/user-impersonation/ |
| docker exec -u <user> <container> <cmd> | flag，先于目标容器 | 身份先于动作目标 | https://docs.docker.com/reference/cli/docker/container/exec/ |
| ssh [user@]host [command] | 身份并入 destination 位置参数 | 紧凑融合语法；另有 -l flag | https://man7.org/linux/man-pages/man1/ssh.1.html |
| psql -U / mysql -u | flag | 连接参数 | https://www.postgresql.org/docs/current/app-psql.html |
三条规律：
1) 主流是"默认身份 + 覆盖 flag"（git/gh/psql），因为这些工具有状态、有默认身份；
2) 无默认身份或显式切换场景，身份出现在被执行的动作之前（sudo/runuser/docker exec），即"actor 先于 action"；
3) 只有 ssh 把身份放进位置参数（user@ 融合）。
=> 业界没有"actor 作为独立必填位置参数"的直接先例，但构成要素（身份先于动作、身份并入位置参数）各有成熟先例；paperwork 的组合是合理创新而非违背惯例。

### 5.2 对 paperwork <file> <name> send 提案的评估
支持理由：
- jmmv 规则（2.1）：actor 在写命令中是必填参数，不应占据 flag 位；
- 身份前置与 sudo/runuser/docker exec 的"actor 先于 action"模式一致（5.1）；
- 文法与 gh/docker 的"名词 scope...动词收尾"连续（2.3）；
- 无状态意味着没有默认身份、必须每次显式；位置参数的简洁性降低重复调用成本（agent 是高频重复调用者，Anthropic）。
风险与对冲：
- 训练数据中该文法频率低，agent 首次接触误调率预计高于 --from flag 形态；必须用 help 示例、SKILL.md、错误自愈三件套补偿（1.6 约束）。
- 位置参数不自描述（Anthropic 参数命名原则针对的正是歧义）；对冲：ok envelope 首行回显 actor（如 ok post.send #3 alice），错误中核验 actor 与 participants 的匹配。
- 只读命令的业界惯例是不要求身份（git log/kubectl get 均如此）；建议写命令（send/edit）必填 actor、读命令（read/summary）可选，形成非对称设计——符合"行动需要身份、观察不需要"的先例（sudo 只在执行时要求 -u）。【可直接采纳：非对称身份】
- Trevin 原则 9（持久 profile）与本提案方向相抵；调和方案：paperwork 的 .profile.md 文件本身就是持久身份的载体，可允许 actor 参数接受"名字或 profile 文件路径"（由 CLI 解析出名字），既保持无状态又减少拼写错误。【仅供参考】
【5.2 总判定】前置位置布局（文件 + actor，写命令）可直接采纳；读命令 actor 可选；ok envelope 回显 actor。

---

## 6. paperwork v0.4 与业界 SOTA 的差距盘点
| SOTA 要求 | 来源 | v0.4 现状 | 差距/动作 |
|-----------|------|-----------|-----------|
| 非交互 | Trevin-1 | 无任何提示 | 无差距 |
| 结构化输出 --json | Trevin-2/Cloudflare/clig | 全命令 --json + envelope | 无差距；--json 错误流需文档声明 |
| actionable 错误 | Trevin-3/Anthropic | fix: + example: 已实现 | 补"枚举合法取值" |
| 安全重试/标识符 | Trevin-4 | send 回执 #N | create 类 already-exists 可附既有信息 |
| 有界响应 | Trevin-5/Infracost | --limit 20 默认 + showing: n/m + summary | 无差距（属前列） |
| 词汇一致性 | Trevin-6/Cloudflare | 动词大体合规 | 无成文命名政策；--from 语义冲突待解 |
| 三层内省 | Trevin-7 | 仅 --help | 缺 SKILL.md 与机器可读 schema |
| 退出码分级 | clig.dev | 统一 1 | 可选增强 |
| 必填参数位置化 | jmmv | PATH 已位置化；actor/--title 等必填仍在 flag 位 | 本次重设计核心 |

---

## 7. 可直接指导本次重设计的可执行结论（按优先级）

C1【可直接采纳】把文件路径 + actor 名字提升为前置必填位置参数（写命令）。依据：jmmv"必填不应是 flag"（2.1）、sudo/runuser/docker exec"身份先于动作"（5.1）、gh/docker"名词 scope...动词收尾"（2.3）。形态：paperwork <file> <actor> send/edit ...
C2【可直接采纳】身份非对称：写操作必填 actor，读操作可选。拒绝非法 actor 时在错误中枚举合法候选（participants/contacts 名单）——Trevin"errors enumerate valid values"。
C3【可直接采纳】消除 --from 语义冲突：actor 提升为位置参数后，post read 的 --from/--to 专用于序列号范围，或改名 --after/--before 以去歧义（Anthropic 参数命名原则）。
C4【可直接采纳】保持默认 envelope + --json + --plain + -q 四档输出：位于"自然语言 vs JSON"争议最优点（1.6），首行结论与 Infracost"summary-before-detail"一致。
C5【可直接采纳】随仓库发布 SKILL.md 并增加机器可读内省（如 paperwork agent-context 或 --help --json）：Infracost 证明 skill 是难题正确率 0/6→6/6 的关键；Trevin 三层内省。
C6【可直接采纳】建立成文命名政策并用测试强制：动词白名单（create/send/read/list/edit/add/remove/verify/validate/summary），--json 为唯一机器 flag，不引入别名；以现有 cli_integration.rs 精确断言模式落地（Cloudflare"Swiss cheese"教训）。
C7【仅供参考】退出码分级（category 映射不同退出码）与 ok envelope 首行回显 actor：低成本增强，agent 不解析文本也能路由。
C8【仅供参考：不采纳】TOON 等新格式：数据量未达收益门槛、生态成本高（1.5d）。
C9【战略确认】继续押注 CLI 形态：paperwork 场景是 CLI 相对 MCP 优势最大区间（1.3x~80x token 差距）；未来若提供 MCP，应为同一功能层的薄门面。
C10【可直接采纳】每个错误维持 category+fix+example 三层，example 永远可复制执行；对前置位置文法，缺参错误尤其要给"一条完整正确命令"。

---

## 8. 风险与异议
1) 前置位置文法在训练数据中频率低，是本次重设计最大风险点。对冲三件套 = help 示例 + SKILL.md + 错误自愈；建议仿 Infracost harness 建小型基准，实测重设计前后的 agent 误调率与 token 成本（1.5 方法论）。
2) 未检索到"位置参数顺序影响 LLM 调用准确率"的同评议研究；现有证据均为工程实测（Infracost/scalekit）与社区讨论。C1 属文法规则 + 先例的理论推导，落地后应实测确认。
3) JSON 错误流（stdout vs stderr）之争无统一结论，任选其一但须显著文档化。
4) Trevin 原则 9（有状态 profile）与 paperwork ADR-011 无状态哲学存在张力；本报告以无状态为约束，不采纳该条。

---

## 9. 来源清单（均于 2026-08-09 访问）
1. Trevin Chow, 7 Principles for Agent-Friendly CLIs — https://trevinsays.com/p/7-principles-for-agent-friendly-clis
2. Trevin Chow, 10 Principles for Agent-Native CLIs — https://trevinsays.com/p/10-principles-for-agent-native-clis
3. Cloudflare, Building a CLI for all of Cloudflare — https://blog.cloudflare.com/cf-cli-local-explorer/
4. Anthropic, Writing effective tools for AI agents — https://www.anthropic.com/engineering/writing-tools-for-agents
5. Infracost, We cut Claude token usage 79% by redesigning our CLI for agents — https://www.infracost.io/resources/blog/we-cut-claude-s-token-usage-79-by-redesigning-our-cli-for-agents
6. ryan, Rethinking CLI interfaces for AI — https://www.notcheckmark.com/2025/07/rethinking-cli-interfaces-for-ai/
7. HN 讨论：Principles for agent-native CLIs — https://news.ycombinator.com/item?id=48052333 ；另见 https://news.ycombinator.com/item?id=48054553
8. clig.dev, Command Line Interface Guidelines — https://clig.dev/
9. Julio Merino, CLI design: Putting flags to good use — https://jmmv.dev/2013/08/cli-design-putting-flags-to-good-use.html
10. StackExchange, Positional arguments vs options — https://softwareengineering.stackexchange.com/questions/366218/
11. Kubernetes, User Impersonation — https://kubernetes.io/docs/reference/access-authn-authz/user-impersonation/
12. git-commit 文档（--author） — https://git-scm.com/docs/git-commit
13. sudo(8) man page（-u） — https://man7.org/linux/man-pages/man8/sudo.8.html
14. docker container exec 参考（-u） — https://docs.docker.com/reference/cli/docker/container/exec/
15. ssh(1) man page（[user@]host） — https://man7.org/linux/man-pages/man1/ssh.1.html
16. scalekit, MCP vs CLI benchmark — https://github.com/scalekit-inc/mcp-vs-cli-benchmark
17. Firecrawl, MCP vs CLI for AI Agents — https://www.firecrawl.dev/blog/mcp-vs-cli
18. TOON 格式 — https://toonformat.dev/ ；https://github.com/toon-format/toon
19. Elm, Compiler Errors for Humans — https://elm-lang.org/news/compiler-errors-for-humans
20. Temporal, Write errors that don't make me think — https://temporal.io/blog/error-message-design
21. Manus 前后端负责人谈 CLI vs 函数调用 — https://www.reddit.com/r/LocalLLaMA/comments/1rrisqn/
22. agentfmt（token-efficient CLI 输出） — https://github.com/dannote/agentfmt
23. gh CLI 手册 — https://cli.github.com/manual/
24. PostgreSQL psql 文档 — https://www.postgresql.org/docs/current/app-psql.html
25. ripgrep — https://github.com/BurntSushi/ripgrep

---
（报告完）
