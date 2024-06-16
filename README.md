## 警告：本项目仅用于学习用途，严禁用于商业用途。
### 作者与贡献者从未参与除开发外任何与GPT-Cat有关的活动，若本项目被应用于除学习外的任何用途，产生的一切后果均与作者及贡献者无关

----
# GPT-Cat
## 介绍
GPT-Cat是一个反向代理服务器，用于管理一系列的LLM后端与用户的交互（如ChatGPT key pool），
并提供一个统一的接口供前端与LLM后端交互。GPT-Cat默认接收OpenAI的ChatGPT格式并返回，开发者可以
自由添加其它后端并进行适配，本项目的初衷是为[NextWeb](https://github.com/ChatGPTNextWeb/ChatGPT-Next-Web)
设计一个后端，来让它同时可以适配多种品牌的LLM。
但任何支持OpenAI的GPT前端都可以通过本项目提供的反向代理功能与任何LLM交互。
本项目完全由Rust开发，您完全可以信任它的性能和安全性。

## 特性
- 🐳 兼容任何支持OpenAI的GPT前端
- 🚀 自由添加任何LLM后端
- 🎢 本地token计算
- 🛩️ 多后端账户池管理
- 🏝️ 用户精细化管理
- 🍭 并发数限制
- 🎨 快捷指令支持
- 📦 一键部署
----

### 添加后端
项目默认提供了[OpenAI](./src/http/client/specific_responder/openai_responder.rs)和[通义千问](./src/http/client/specific_responder/qianwen_responder.rs)的适配，因此你可以参考这两个适配器来添加你自己的适配器。  
- "Endpoint"： 支持的后端列表，在适配新后端时，应首先在此处添加你的后端信息，并根据rustc提供的信息完成相关信息的填写。
- "SpecificResponder"：适配器的基本结构，在其中完成你的请求，并返回一个Response。
- "ResponseParser"：内置的解析工具，可以解析SSE信息，实现它后可以在其中解析你的请求。

### 预处理器
预处理器是一个在请求到达后端之前的处理器，请查看[这里](./src/http/server/mod.rs)以了解更多信息。
总的来说，这是一个用于过滤请求的管道，在客户端发起请求后，请求会被预处理器处理，在确认请求的合法性后,继续执行后续的操作。
项目默认提供了如下预处理器：
- "ModelFilterHandler": 用于过滤请求中的model，如果用户请求的model不在后端的model列表中，则会被拒绝。
- "UserKeyHandler": 用于提取用户请求头中包含的key，如果用户请求头中没有key，则会被拒绝。
- "UserIDHandler": 将用户的key转换为用户的id，如果用户的key不在后端的key列表中，或者用户的key处于冻结状态，则会被拒绝。
- "TitleCatchHandler": 来源于NextChat中的一个bug:　如果使用的模型不是gpt系列模型，会使用默认的模型来生成对话标题，这个迷惑行为最初给我的claude额度来了一记重拳，这个预处理器用于捕获这个标题并强制修改为gpt3.5
- "CommandHandler"：捕获快捷指令，并按照对应的模板进行展开

### 后处理器
后处理器是一在请求完成后执行的处理器，请查看[这里](./src/http/server/mod.rs)以了解更多信息。
它通常被用于进行请求后的额外处理，比如账户计费，记录日志等。
项目默认提供了如下后处理器：
- "TokenMeterHandler": 用于计算用户的token消耗，并在数据库中进行相应的扣除操作。