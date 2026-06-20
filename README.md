# TPush

端到端加密的实时消息推送系统，Android 客户端接收推送消息，Rust 后端提供服务，Web 管理面板用于发送消息。

## 项目结构

```
push_test/
├── app/                    # 移动端 + Web 面板
│   ├── src/                # React Native Android 应用
│   │   ├── App.tsx         # 主应用组件
│   │   └── components/     # UI 组件（PushAppBar、MessageList 等）
│   ├── core/               # Rust 核心库（JNI 绑定）
│   ├── panel/              # SolidJS Web 管理面板
│   └── android/            # Android 原生工程
├── server/                 # Rust Axum 后端服务
│   └── src/
│       ├── main.rs         # 服务入口
│       ├── routes.rs       # HTTP API（推送、同步）
│       ├── channel_routes.rs # 频道管理
│       └── db.rs           # SQLite 数据库
├── scripts/                # 构建/运行脚本
└── output/                 # 构建产物
```

## 技术栈

| 模块 | 技术 |
|------|------|
| Android 客户端 | React Native 0.76 + TypeScript + Material Design 3 |
| 原生核心库 | Rust（JNI 绑定，UniFFI） |
| Web 管理面板 | SolidJS + Vite + TypeScript |
| 后端服务 | Rust + Axum + SQLite + WebSocket |
| 消息加密 | AES-256-GCM + HMAC-SHA256 签名 |
| 构建系统 | pnpm + Turborepo + Cargo |

## 快速开始

### 环境要求

- Node.js >= 18
- pnpm >= 9
- Rust toolchain（stable）
- Android SDK（用于构建移动端）
- Java 21（Android 编译）

### 安装依赖

```bash
pnpm install
```

### 启动后端服务

```bash
pnpm run dev:server
```

服务默认监听 `http://0.0.0.0:3000`。

### 启动 Web 管理面板

```bash
pnpm run dev:panel
```

面板默认运行在 `http://localhost:5173`，API 请求自动代理到后端 `127.0.0.1:3000`。

### 构建 Android 应用

```bash
# Debug 构建
pnpm run build:debug

# Release 构建
pnpm run build
```

构建产物输出到 `output/` 目录。

### 本地开发运行

```bash
# Debug 模式（启动服务 + 安装应用）
pnpm run local

# Release 模式
pnpm run local:release
```

## 功能

- **端到端加密** — 消息在发送端使用 AES-256-GCM 加密，仅持有正确密钥的客户端可解密
- **多频道支持** — 支持多频道消息隔离，不同频道使用不同密钥
- **持久化存储** — 消息存储在本地 SQLite，支持离线查看
- **实时推送** — 基于 WebSocket 的长连接，消息到达即时通知
- **Web 管理面板** — 浏览器中发送消息、管理频道
