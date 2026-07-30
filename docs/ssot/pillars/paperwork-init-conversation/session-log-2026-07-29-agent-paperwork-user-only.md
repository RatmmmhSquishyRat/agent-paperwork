# Session Log (User Messages Only) — Agent Paperwork Design Discussion

**Date**: 2026-07-29  
**Purpose**: Owner's design directives and decisions for `agent paperwork`.

---

## 1

你如何理解基于文件的agent上下文渐进式披露?

---

## 2

如果我把一些常用的经常一起阅读的文件, 通过cli输出一次性给到agent, 这对于长上下文模型来说, 是反模式还是有益模式?

---

## 3

好吧, 我说说我想做的东西 - 我想做的是一个agent之间基于文件的合作util toolkit cli, 叫做 agent paparwork:
几个主要功能: 
1. 完全基于文件的agent role profile, 描述自己是谁, 有什么模型配置, 将会干什么, 以及最重要的涉及读写文件范围和只读文件范围.
2. 完全基于文件的通讯录(各个profile以及对应私聊文件夹的路径), 私聊(DM)和群聊(meeting vis GDM), 全append无阻碍编辑.
3. 完全基于文件路径列表(也支持相对路径和blob, 乃至行号/hashline范围)的, 一个快速阅读文件清单. 不再需要agent渐进式披露, agent可以自己总结固化阅读的最佳实践, 落盘到read manifest 文件, 让其他agent能正确快速掌握需要理解的文本文件. agent读取时候会先读到目录, 然后可以选择直接全量阅读, 或者根据路径自己手动选择性阅读.

这些基本上是collaboration软件中我认为对于agent来说最有价值和值得做的部分. 

你如何理解这个项目, 你认为是否有任何我没有想到的地方, 或者需要改进的?

---

## 4

1. agent不需要显式记录自己读到哪里, 因为DM文件本来就是append语义的, 而tool call又是agent自己产生的, 所以它只需要接着上次的msg序号往下读就好了. 并且, GDM还可以被读取状态概要, 标题, 一共多少个msg, 上次更新时间和用户, 各个msg的摘取速览等等. 因此, agent在读取具体消息时候, 需要做的只有指定序号范围就够了, 和读取文件行号范围是类似的.
1.5. 我想到DM中应该允许agent@别的profile, 这些@消息会被记录于profile中, 并通过cli提醒主人, 提醒后归入历史通知记录, 存储于在单独文件中.
1.75. 并且, 这里我还想到可以根据@profile或者reply的消息来筛选查看相关message, 然后reply默认带有@, 和主流DM应用一致.
2. 所有这个cli相关的文件, 都是managed, 这属于core负责的机制.
3. 我的建议是, 既不是行号, 也不是hashline, 而是实现正则表达式抽取文字, 甚至允许多组匹配输出. 失败情况下, 则就可以知道文件被更新; 另外, 确实有可能改变但是失败静默, 或者预料内持续成功, 这种情况下文件版本应当单独建模, 也就是在具体路径后, 记录blob path的hash即可, 调用时候校验即可.
4. cli不会也无法enforce读写范围, 所以profile中的范围都是自主负责, 动态协商调整的.
5. DM只是append onlu, 无法进行插入和删除而已, 而这两种操作在工单中实际上都是污染. 因此, 有append only, 有@, 有编辑自己消息能力, 这些功能足够工单使用了, 只不过是cli harness层级没有语义说明罢了.
6. 有道理, paperwork应当支持查询指定路径范围A内的profile谁读写/只读/owns(这个语义不错)指定路径范围内的文件.
7. 这要看你到底关注哪一点, 是可读性还是体积, 文件太大需要全部归档, agent自己来操作就行; 可读性不好, 可以让post owner/users给summary, 尤其是过长的消息, 过长的消息序列段, 给summary就行.
8. 有道理, 先流畅轻松甚至stub创建, 然后再细致编辑, cli的全程UX都应该遵循这种体验.

我觉得你提的殿都很有道理, 因此上述我进行了完整的补全.

---

## 5

1. There is no core loop in design, the DM, the profile, the manifest-baed brief, they can be all relevant to each other, but no hierarchy, no fixed usage loop, each forms a individual usage tool themselves.
2. To be a owner, I wont down into implementation details, protocols or whaterver, at most just adrs. I write ssot and pillars, I play your delivery and then give feedback, that will be the primary intervention form.

Now generate a session log file with all messages verbatimly, losslessly, as an ssot conversation record.

---

## 6

再给一个纯用户消息版本的

---

*End of session log.*
