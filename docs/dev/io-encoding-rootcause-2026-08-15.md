# io 信封中文 OS 错误消息乱码 — 根因分析报告

- 日期：2026-08-15
- 任务：#25（诊断 + 文档，本阶段不改源代码）
- 分析对象：worktree `agent-paperwork-wt-v06grammar`（分支 `cli-grammar-v0.6`）
- 登记来源：contacts CRUD 轮 Kim 正确性评审 M-2；v0.6 轮类似记录
- 环境：Windows 中文系统（zh-CN，OEM 代码页 936 / GBK），PowerShell 7

## 1. 结论判定（钉住）

**这是捕获端解码层的环境问题，不是产品输出字节层的缺陷。产品无需代码修复。**

- 产品（paperwork CLI）在**所有输出模式**（default / `--json` / `--plain`）、**所有流**（stdout / stderr）、**所有 quiet 组合**下，写出的字节均为**合法 UTF-8**，经字节级取证逐一验证（见 §3）。
- 乱码发生在下游捕获端：PowerShell 7 在中文系统上默认 `[Console]::OutputEncoding = gb2312 (cp936)`，当其经管道捕获原生命令的 **stderr**（`2>` 重定向或 `$x = cmd 2>&1`）时，把产品的合法 UTF-8 字节**按 GBK 误解码**成字符串，落盘/回显时再编码为 UTF-8，形成**双重编码乱码**（mojibake）。
- 乱码字节可由「UTF-8 字节 → GBK 解码 → UTF-8 再编码」程序化复现，与实测捕获字节**逐字节一致**（含全角句号「。」不可映射退化为 `0x3F`「?」），机制完全闭合。
- cmd.exe 文件重定向、`Start-Process -RedirectStandard*`、以及把 `[Console]::OutputEncoding` 设为 UTF-8 后的 pwsh 捕获，均得到干净 UTF-8 —— 证明字节源头无损。

修复轮已把三处 io `fix` 文案回填为英文基线原文（"check that the target path is writable"），登记症状（fix 字段乱码）已消除。本报告将该问题正式钉住为「环境侧」，并给出 agent 消费端规避方案（§5）与文档声明建议（§6）。

## 2. 错误消息的因果链（Rust 侧静态定位）

1. OS 报错：Windows `FormatMessageW` 返回 UTF-16 本地化消息（中文系统即「系统找不到指定的路径。」），Rust 标准库转为 UTF-8 `String` 装入 `std::io::Error` —— **此步产出即合法 UTF-8**。
2. 错误包装：`paperwork-core/src/ops/{lock,profile,manifest,contacts}.rs` 与 `paperwork-cli/src/cmd/{post,validate}.rs` 将其包为 `PaperworkError::IoContext { source, fix, .. }`（fix 现为英文基线文案）。
3. 消息渲染：`paperwork-core/src/error.rs` L29 `#[error("IO error at '{}': {source}", path.display())]` —— **message 字段因此内嵌中文 OS 文本**（fix 字段不含）。
4. 信封分发：`paperwork-cli/src/main.rs` L129–153 → `output.rs::emit_err`：
   - `--json`：`serde_json::to_string`（恒输出合法 UTF-8，非 ASCII 原样嵌入）→ **stdout**，含 `exit_code` 字段；
   - default：`eprintln!` 三行信封 → **stderr**。

即：中文只出现在 `message` 字段；`fix`/`example` 字段自修复轮起恒为英文。

## 3. 复现实验与字节证据

构建：`cargo build -p paperwork-cli`（worktree，debug）。触发命令：
`paperwork validate C:\nonexistent-dir-zz\no.md --type post`（io 类，os error 3）。
中文基准串「系统找不到指定的路径。」的合法 UTF-8 字节 =
`E7 B3 BB E7 BB 9F E6 89 BE E4 B8 8D E5 88 B0 E6 8C 87 E5 AE 9A E7 9A 84 E8 B7 AF E5 BE 84 E3 80 82`。

| # | 捕获方式 | 中文段字节 | 判定 |
|---|---|---|---|
| a | pwsh `> file`（JSON stdout，cp936 会话） | `E7 B3 BB E7 BB 9F … E3 80 82`（基准原文） | ✅ 干净 UTF-8 |
| b | pwsh `2> file`（default stderr，cp936 会话） | `E7 BB AF E8 8D A4 E7 B2 BA E9 8E B5 … 3F`（222 字节） | ❌ 双重编码乱码 |
| c | cmd.exe `.bat` 内 `>` / `2>` 重定向 | stdout、stderr 均 = 基准原文 | ✅ 干净 UTF-8 |
| d | cmd.exe `chcp 65001` vs `chcp 936` | 两者重定向字节**完全一致**（均基准原文，204 字节） | ✅ chcp 不影响重定向字节 |
| e | `Start-Process -RedirectStandardOutput/-Error` | stdout、stderr 均 = 基准原文 | ✅ 干净 UTF-8 |
| f | 最小探针（println!/eprintln! 同一中文串） | stdout 两环境干净；stderr 仅 pwsh-cp936 乱码 | ✅ 排除产品特殊性 |
| g | pwsh 会话内设 `[Console]::OutputEncoding=UTF8` 后 `2>` | stderr = 基准原文（40 字节） | ✅ 规避手段有效 |
| h | `$x = paperwork … 2>&1`（agent 常用变量捕获，cp936） | 3 个 ErrorRecord，中文段乱码；UTF-8 模式后干净 | ❌→✅ 正是登记症状现场 |
| i | 机制复现：`UTF8.GetBytes` → `GBK.GetString` → `UTF8.GetBytes` | 输出与 b 的乱码字节**逐字节一致** | ✅ 机制闭合 |

JSON 信封实测（pwsh 捕获 stdout，UTF-8 校验通过，286 字节）：

```json
{"category":"io","command":"validate","example":"paperwork validate C:\\nonexistent-dir-zz\\no.md","exit_code":1,"fix":"check that the file exists and is readable","message":"IO error at 'C:\\nonexistent-dir-zz\\no.md': 系统找不到指定的路径。 (os error 3)","status":"error"}
```

附带核查结论：

- **JSON 输出恒 UTF-8**：`serde_json::to_string` 序列化非 ASCII 字符原样嵌入 UTF-8，实测全部通过 `UTF8Encoding(throwOnInvalidBytes=true)` 校验。
- **`--quiet` 与 JSON 一致**：`--quiet --json` 与 `--json` 输出字节完全一致（286 字节）；quiet 只抑制 default 模式成功状态行，不影响错误信封。
- **JSON 模式错误走 stdout**（含 `exit_code`），default 模式错误走 stderr —— agent 在 default 模式下必须捕获 stderr 才能拿到信封。
- pwsh 存在 stdout/stderr 捕获不对称（cp936 下 stdout 干净、stderr 乱码）：stderr 被 PowerShell 包装为 ErrorRecord 走独立解码路径。此不对称是 PowerShell 行为，与本产品无关。

## 4. 复现命令（可重放）

```powershell
$exe = '<worktree>\target\debug\paperwork.exe'

# 症状复现（cp936 会话，默认即如此）：stderr 捕获得乱码
& $exe validate C:\nonexistent-dir-zz\no.md --type post 2> err.bin
$cap = & $exe validate C:\nonexistent-dir-zz\no.md --type post 2>&1   # ErrorRecord 乱码

# 干净捕获（三选一）
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8               # 方式1：会话内钉住 UTF-8
cmd /c "$exe validate ... 2> err.bin"                                  # 方式2：cmd 重定向
Start-Process $exe -ArgumentList ... -RedirectStandardError err.bin -Wait  # 方式3：.NET 重定向

# 字节校验
$b = [IO.File]::ReadAllBytes('err.bin')
[System.Text.UTF8Encoding]::new($false, $true).GetString($b)           # 抛异常即非法 UTF-8
```

## 5. agent 消费端规避建议

1. **首选**：捕获前执行 `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8`（需要发给原生命令中文输入时同时设 `$OutputEncoding`）。
2. 会话级备选：控制台 `chcp 65001`（注意：chcp 只影响控制台显示与 pwsh 初始化编码，不改变已重定向的字节）。
3. 结构化消费优先用 `--json`：错误信封随 stdout 输出、无编码歧义、可被 `ConvertFrom-Json` 直接解析。
4. 文件落盘消费时按 UTF-8 解码（`Get-Content -Encoding utf8` / `[IO.File]::ReadAllText($p, [Text.Encoding]::UTF8)`）。

## 6. 产品侧评估与文档建议

**评估结论：产品侧不做代码修复。** Rust 程序无法控制已重定向字节的下游解码；产品字节恒为合法 UTF-8，无可修之缺陷。修复轮的英文 fix 文案回填保留（它同时消除了乱码现场最刺眼的症状，且属稳定英文契约）。

**残留面**：`message` 字段仍内嵌 OS 本地化文本（中文系统为中文），在 cp936 捕获端仍会呈现乱码。两个可选方向，均**不建议在本轮实施**，留作 backlog 评估：

- （文档契约，建议采纳）在 SKILL.md / README 声明：*paperwork 的所有输出（stdout 与 stderr，全部模式）恒为 UTF-8；消费端必须以 UTF-8 解码，Windows PowerShell 会话请先设置 `[Console]::OutputEncoding = [Text.Encoding]::UTF8`。*
- （代码硬化，可不做）将 `error.rs` IoContext 的 Display 中 `{source}` 替换为与区域无关的形式（如仅保留 `os error N` 错误码），彻底消除输出中的本地化文本；代价是损失 OS 原文信息量，且与「信封携带可读错误上下文」的设计取向冲突。

## 7. 证据链摘要

1. 静态：io::Error → IoContext → thiserror Display（中文进 message）→ emit_err（JSON→stdout / default→stderr）——代码路径完整。
2. 动态：6 种捕获方式字节取证，产品源头 3 条独立路径（pwsh 直重定向、cmd、Start-Process）全部干净 UTF-8。
3. 机制：双重编码推导字节与乱码捕获逐字节一致；UTF-8 编码设置后症状消失（充分必要双向验证）。
4. 附带：JSON 恒 UTF-8、`--quiet`/JSON 一致性、chcp 对重定向字节无影响 —— 均实证。
