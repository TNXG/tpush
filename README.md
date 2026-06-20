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

### 启动服务（含管理面板）

```bash
# 1. 构建管理面板静态文件
pnpm run build:panel

# 2. 启动后端（面板已内嵌在二进制中）
cargo run
```

服务默认监听 `http://0.0.0.0:3000`，管理面板直接访问该地址即可。

> 首次启动会自动创建 SQLite 数据库文件并执行表迁移。
>
> 如需配置监听地址或登录凭据，复制 `config.example.toml` 为 `config.toml` 并修改。

### 开发模式

```bash
# 后端热重载
cargo watch -x run

# 面板独立开发（Vite HMR，API 代理到 3000）
pnpm run dev:panel
```

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
- **Web 管理面板** — 内嵌于后端的 SolidJS 管理面板，JWT 鉴权

## 配置文件

服务启动时读取 `config.toml`（可通过 `CONFIG_PATH` 环境变量指定路径）：

```toml
[server]
bind_address = "0.0.0.0:3000"

[auth]
jwt_secret = "change-me-to-a-random-secret"
username = "admin"
password = "admin123"
```

---

## API 参考

### 频道模型

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 频道名称，全局唯一，仅允许字母数字及 `-` `_`，最长 80 字符 |
| `key` | string | 频道密钥。非空时频道为**私有**，消息加密下发且客户端需签名；为空时频道为**公开** |

**私有频道签名机制：** 客户端调用需鉴权的端点时，携带 `ts`（时间戳）、`nonce`（随机数）、`signature`（HMAC-SHA256 签名）。签名原文为 `channel:subject:ts:nonce`，密钥为频道 key 原始值。

### 消息模型

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 消息 UUID |
| `channel` | string | 所属频道 |
| `title` | string | 通知标题 |
| `content` | string | 消息正文 |
| `extras` | object | 自定义扩展数据（JSON） |
| `delivery_status` | string | 投递状态：`queued` 或 `online_sent:N` |
| `created_at` | string | ISO 8601 创建时间 |

消息加密信封 `EncryptedEnvelope`：

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | int | 协议版本，当前为 1 |
| `channel` | string | 频道 |
| `algorithm` | string | 加密算法：`AES-256-GCM+SHA256` 或 `none` |
| `encrypted` | bool | 是否加密 |
| `data` | string | 密文（Base64，含 12 字节 nonce 前缀）或明文 |

---

### 接口列表

#### 设备注册

```
POST /api/devices/register
鉴权：私有频道需签名
```

**请求：**
```json
{
  "deviceId": "android-uuid",
  "channel": "default",
  "auth": {
    "ts": "1703001234",
    "nonce": "random-hex",
    "signature": "hmac-sha256-hex"
  }
}
```

**响应：**
```json
{ "id": "device-uuid" }
```

---

#### WebSocket 实时流

```
GET /api/channels/:channel/stream?ts=&nonce=&signature=
鉴权：私有频道需签名（query 参数）
```

升级为 WebSocket 长连接，服务端以 `EncryptedEnvelope` JSON 文本帧推送消息。

- **私有频道**：帧内容为加密信封（`encrypted: true`），数据经 AES-256-GCM 加密后 Base64 编码，客户端需用频道 key 解密
- **公开频道**：帧内容为明文信封（`encrypted: false`），`data` 直接为消息 JSON

客户端无上行消息（仅维持连接）。

---

#### 发送推送

```
POST /api/push
鉴权：无（公开接口）
```

**请求：**
```json
{
  "channel": "default",
  "title": "新通知",
  "content": "消息内容",
  "extras": {}
}
```

**响应：**
```json
{
  "id": "message-uuid",
  "accepted": true,
  "online_deliveries": 3
}
```

`channel` 默认为 `"default"`，`extras` 默认为 `{}`。推送会同时写入数据库并广播给当前在线客户端。

---

#### 消息列表

```
GET /api/messages
鉴权：无（公开接口）
```

**响应：** 最近 200 条消息，按创建时间倒序。

```json
[
  {
    "id": "msg-uuid",
    "channel": "default",
    "title": "标题",
    "content": "正文",
    "extras": "{}",
    "delivery_status": "online_sent:3",
    "created_at": "2024-12-20T10:30:00Z"
  }
]
```

---

#### 删除消息

```
DELETE /api/messages
鉴权：无（公开接口）
```

**请求：**
```json
{ "ids": ["msg-uuid-1", "msg-uuid-2"] }
```

**响应：**
```json
{ "deleted": 2 }
```

单次最多删除 500 条。

---

#### 消息同步

```
GET /api/messages/sync?channel=&deviceId=&ts=&nonce=&signature=&after=
鉴权：私有频道需签名
```

客户端离线后拉取历史消息，返回 `EncryptedEnvelope[]`。

| 参数 | 说明 |
|------|------|
| `channel` | 频道名 |
| `deviceId` | 设备 ID |
| `ts` / `nonce` / `signature` | HMAC 签名参数 |
| `after` | 可选，ISO 8601 时间戳，只拉该时间后的消息（升序） |

不带 `after` 时返回最近 200 条（升序排列）。

---

#### 频道列表

```
GET /api/channels
鉴权：JWT（管理面板）
```

**响应：**
```json
[
  {
    "id": "channel-uuid",
    "name": "default",
    "key": "my-secret-key",
    "created_at": "2024-12-20T10:00:00Z",
    "updated_at": "2024-12-20T10:00:00Z"
  }
]
```

---

#### 创建/更新频道

```
POST /api/channels
鉴权：JWT（管理面板）
```

**请求：**
```json
{
  "name": "my-channel",
  "key": "my-secret-key"
}
```

**响应：** 返回完整的 `ChannelItem`。`key` 留空即为公开频道。频道已存在时更新密钥。

---

#### 删除频道

```
DELETE /api/channels/:channel
鉴权：JWT（管理面板）
```

级联操作：删除频道的所有消息，将该频道的客户端设备移至 `default` 频道，断开 WebSocket 连接。

**响应：**
```json
{
  "deleted_channel": true,
  "deleted_messages": 42
}
```

---

#### 管理面板登录

```
POST /api/admin/login
Content-Type: application/x-www-form-urlencoded
鉴权：无
```

**请求：**
```
username=admin&password=admin123
```

**响应：**
```json
{ "token": "eyJhbGciOiJIUzI1NiJ9..." }
```

Token 有效期 24 小时。管理面板 API 需携带 `Authorization: Bearer <token>` 请求头。

---

### 鉴权总结

| 端点 | 鉴权方式 |
|------|---------|
| `POST /api/push` | 无 |
| `GET /api/messages` | 无 |
| `DELETE /api/messages` | 无 |
| `POST /api/devices/register` | 私有频道：HMAC 签名 |
| `GET /api/channels/:channel/stream` | 私有频道：HMAC 签名 |
| `GET /api/messages/sync` | 私有频道：HMAC 签名 |
| `GET /api/channels` | JWT（管理面板） |
| `POST /api/channels` | JWT（管理面板） |
| `DELETE /api/channels/:channel` | JWT（管理面板） |
| `POST /api/admin/login` | 无 |
