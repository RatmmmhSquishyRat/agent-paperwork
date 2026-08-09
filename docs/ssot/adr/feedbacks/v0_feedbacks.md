# v0 feedback

首先我要说最重要的: 在这个项目创建之初, 我就不想要把这个项目的数据做成一个.paperwork的托管文件夹形式 - 我说的托管文件(managed file), 是针对于单个, 或者单个小组文件而言的, 不是直接把所有文件都放到托管文件夹中. 这个cli, 要做到, 在任意路径启动, 都能够对于任意路径的格式匹配的文件进行操作管理, 保证最大的任意场景临场可用性, 做到尽可能的无状态, ssot全部来自于操作的文件. 而各个文件之间, 也不应该通过managed center进行互相引用, 而最多就是agent主动自定义的在内容中写引用路径(责任范围外).

其次, 这个cli, 没有传统意义上的登录语义, 大部分harness就不支持登录之后持续命令输入, 而这个cli的各种操作本来也不需要给登录身份, GDM给名字就够了.

因此, 我在这里显式给出ADR, 那就是这个cli manage范围内的各个文件之间, **没有默认依赖cli机制的硬性连接**:

- GDM中的消息只有身份&内容&(标题&简述), 没有profile地址等等信息, 理论上谁都可以造假身份来编辑消息
- profile中没有所谓私聊地址, profile对应的私聊文件夹, 属于一个managed的profile的附加文件夹, 就创建在profile同目录, 名字也managed. 每个DM文件指定聊天另一方名称, 就这么简单.
- contact就是一个特殊的brief, 从cli读的时候, 显示各个路径加上各个路径上的profile简介.
- brief根本就和身份profile没有半毛钱关系, 最多给一个owner name.

因此, 没有所谓.paperwork文件夹. 有的就是能够通过cli一个个创建文件, 修改文件, 使用文件, 仅此而已.

## 追加feedback

的确使用追随profile的文件来存储DM是一个糟糕的做法, GDM本身已经能够涵盖DM的功能了, 因此DM应当删除.

## v0.2 feedback

1. 管理的各个文件都应当使用自己的sub suffix, 类似 impl1.profile.md, meeting1.post.md等等
2. 既然我们选择使用md作为文件格式, 那么就以正规简洁的方式组织信息结构, 严谨克制, 但是自由灵活地使用各个标题, 列表等等语法.
3. 而矛盾点在于, 使用者输入的内容也需要是markdown格式. 因此:
   1. 输入的时候, 把content类型的参数放在最后方便书写有换行的大片内容
   2. 输入之后给一个markdown validation机制帮助检查markdown语法是否正确
   3. 在我们的managed文件中, 以fenced code block形式包裹, 并设置为markdown block, 这样就能够让文件支持多层markdown了.

> 注记（2026-08-09 追加，不改写原文）：v0.2 feedback 第 3 条之 3.1 字面条款（content 类型参数放在最后）已于 2026-08-09 被 owner v0.6 指令翻转，正文改经 `--message`/`--stdin` 具名传递，书写便利精神保留；见 `docs/ssot/adr/feedbacks/v0.6_feedbacks.md` §3.1（Nora ISSUE-m1 补链）。
