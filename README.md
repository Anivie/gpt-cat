## 警告：本项目仅用于学习用途，严禁用于商业用途。
### 作者与贡献者从未参与除开发外任何与GPT-Cat有关的活动，若本项目被应用于除学习外的任何用途，产生的一切后果均与作者及贡献者无关

----
# GPT-Cat
## 介绍
GPT-Cat是一个反向代理服务器，用于管理一系列的LLM后端与用户的交互（如ChatGPT key pool），
并提供一个统一的接口供前端与LLM后端交互。GPT-Cat默认接收OpenAI的ChatGPT格式并返回，开发者可以
自由添加其它后端并进行适配，本项目的初衷是为[NextWeb](https://github.com/ChatGPTNextWeb/ChatGPT-Next-Web)
设计一个后端，来让它同时可以适配多种品牌的LLM，但实际上任何支持OpenAI的GPT前端都可以通过本项目提供的反向代理功能与任何LLM交互。  
你可能注意到这是一个全新的仓库，这是因为这个项目在设计之初被用于一些难以开源的目的，但随着需求的不断变动，它已经渐渐成为了一个普通的组件，因此我决定将它开源，希望它能够帮助到更多的人。  
这并不意味着这个项目仍处于不稳定的快速开发阶段，实际上，它已经为我和我的朋友们提供了近半年的服务，因此，我相信它已经足够稳定，但你在将它应用于生产环境前，我仍然建议您做好足够的测试和准备工作。

## 特性
- 🐳 兼容任何支持OpenAI的GPT前端
- 🚀 自由添加任何LLM后端
- 🎢 本地token计算
- 🛩️ 多后端账户池管理
- 🍭 后端请求并发数限制
- 🏝️ 用户精细化管理
- 🎨 快捷指令支持
- 📦 一键部署

本项目完全由Rust开发，您完全可以信任它的性能和安全性。

### 多后端兼容
GPT-Cat支持多种后端，您可以自由添加您的后端，并通过适配器来适配新的后端，默认提供ChatGPT和通义千问的适配器。

### 账户池管理
可在单数据库中存放多种后端的key，轻松管理账户池。

### 用户精细化管理
默认提供额度管理，计划提供用户组支持

### 快捷指令支持
支持快捷指令，可以在请求中使用快捷指令来快速进行问答，默认支持translate等指令，可帮助LLM稳定提供回答。

----
## 展示

![HELP](./.github/readme_image/builtin_help.png)  
预置的指令，可以使用help查看所有支持的指令

![HELP](./.github/readme_image/template_help.png)
预置的模板，可以自由修改

![模板使用](./.github/readme_image/template1.png)  
使用模板进行对话

![模板使用](./.github/readme_image/template2.png)  
模板可以插入任意位置，方便进行对话

----
## 快速开始
- 安装[Docker Compose](https://docs.docker.com/compose/install/)
- 下载Compose[配置文件](./docker-compose.yaml)
- 配置[环境变量](./src/data/config/config_helper.rs)
- (启用HTTPS) 将包含`fullchain.pem`和`key.pem`的`ssl`文件夹挂载到`/app/`目录下
，检测到`ssl`文件夹后，GPT-Cat会自动启用HTTPS
- 运行`docker compose up`启动服务

## 二次开发
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