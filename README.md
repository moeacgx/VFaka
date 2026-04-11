# AFF Card Shop (发卡商城)

自动发卡商城系统 — Rust (Actix-web) + Vue 3 + TailwindCSS

## 功能特性

- **自动发卡**: 用户购买后自动发放卡密，邮箱查询订单
- **管理后台**: 商品/分类/卡密/订单 CRUD，批量导入补货，仪表盘统计
- **双支付通道**: 易支付 (支付宝/微信/QQ) + TokenPay (USDT/TRX 加密货币)
- **AFF 推广系统**: 邀请返佣，多级等级体系 (青铜→白银→黄金→钻石)，自动升级
- **多链提现**: 支持 USDT/USDC 在 Tron、Polygon、Base 链提现
- **支付后触发**: 支持 Webhook 回调和 Shell 命令执行
- **商品媒体**: 商品支持上传图片/视频，本地或 S3 存储
- **商城公告**: 管理员可配置公告栏，支持 info/warning/success 样式
- **Telegram 通知**: 支付成功和提现申请实时推送到 Telegram Bot
- **邮件通知**: SMTP 邮件通知订单确认和提现状态变更
- **多语言**: 简体中文、繁体中文、English，浏览器自动检测
- **深色模式**: 跟随系统 / 手动切换 (亮色/暗色/自动)
- **模块化架构**: Cargo Workspace 7 个 crate，清晰分层

## 快速开始

### Docker 部署 (推荐)

#### SQLite (默认)

```bash
# 1. 创建配置文件
cp config.toml config.local.toml
# 编辑 config.local.toml，修改 JWT secret 和管理员密码

# 2. (可选) 配置 TokenPay 加密货币支付
cp config/tokenpay/appsettings.json.example config/tokenpay/appsettings.json
# 编辑 appsettings.json，填写收款地址

# 3. 创建数据目录
mkdir -p data data/uploads

# 4. 启动 (包含 TokenPay)
docker compose up -d

# 仅启动商城
docker compose up -d app

# 查看日志
docker compose logs -f
```

#### PostgreSQL

```bash
# 使用 PostgreSQL compose overlay
docker compose -f docker-compose.yml -f docker-compose.postgres.yml up -d

# 修改数据库密码: 编辑 docker-compose.postgres.yml 中的
#   POSTGRES_PASSWORD 和 AFF_DATABASE_URL

# 查看日志
docker compose -f docker-compose.yml -f docker-compose.postgres.yml logs -f
```

### 本地开发

**环境要求:** Rust 1.88+, Node.js 18+, pnpm

```bash
# 1. 构建前端
cd frontend && pnpm install && pnpm run build && cd ..

# 2. 运行服务器 (默认 SQLite)
cargo run

# 使用 PostgreSQL
cargo run --features postgres --no-default-features
```

服务器启动在 `http://127.0.0.1:8080`

- 前台商城: `http://127.0.0.1:8080/`
- 管理后台: `http://127.0.0.1:8080/admin`
- 默认管理员: `admin` / 首次启动时控制台输出的随机密码 (请及时保存)

### 配置

复制并修改配置文件:
```bash
cp config.toml config.local.toml
```

```toml
[server]
host = "127.0.0.1"
port = 8080
# Required for production — used for CORS and callback URLs
# public_base_url = "https://your-domain.com"
# Optional: explicit CORS origins (defaults to public_base_url + local)
# allowed_origins = ["https://your-domain.com"]

[database]
# SQLite (default)
url = "sqlite:./aff_shop.db?mode=rwc"
# PostgreSQL — build with --features postgres --no-default-features
# url = "postgres://user:password@localhost:5432/aff_shop"

[jwt]
# MUST change before production — server will refuse to start with default secret
# when public_base_url is set
secret = "your-jwt-secret-change-this"
expiration_hours = 24

[admin]
username = "admin"
# If left as "admin123", a random password is generated on first run (check logs)
password = "your-secure-password"
```

## 支付通道配置

### 易支付 (pay.myzfw.com)
在管理后台 → 支付通道 → 易支付，填写:
- 商户 ID (pid)
- 商户私钥 (RSA)
- 平台公钥 (RSA)
- API 地址 (默认 https://pay.myzfw.com)

### TokenPay
1. 部署 [TokenPay](https://github.com/LightCountry/TokenPay) 服务 (使用 `docker compose` 会自动部署)
2. 编辑 `config/tokenpay/appsettings.json`，填写收款钱包地址
3. 在管理后台 → 支付通道 → TokenPay，填写:
   - API 地址 (Docker 内: `http://tokenpay:5000`)
   - 通知密钥 (与 appsettings.json 中的 ApiToken 保持一致)

## 通知配置

### Telegram Bot
1. 通过 [@BotFather](https://t.me/BotFather) 创建 Bot，获取 Token
2. 获取 Chat ID (群组或个人)
3. 管理后台 → 设置 → Telegram 区域填写 Bot Token 和 Chat ID
4. 点击 "测试" 验证

### SMTP 邮件
1. 管理后台 → 设置 → SMTP 区域填写邮箱服务器信息
2. 支持: QQ 邮箱、Gmail、自建 SMTP 等
3. 点击 "测试" 发送测试邮件

## 存储配置

商品图片/视频支持两种存储后端:

- **本地存储** (默认): 文件保存在 `data/uploads/`，通过 `/uploads/` 路径访问
- **S3 存储**: 兼容 S3 协议的对象存储 (AWS S3、MinIO、Cloudflare R2 等)

在管理后台 → 设置 → 存储区域切换和配置。

## 项目结构

```
AFF/
├── crates/
│   ├── aff-common/    # 共享: 配置、错误、类型、加密、ID 生成
│   ├── aff-entity/    # 数据层: SeaORM Entity + Migration + DTO
│   ├── aff-payment/   # 支付: PaymentProvider trait + 易支付/TokenPay
│   ├── aff-core/      # 业务逻辑: 订单、卡密、AFF、提现等 Service
│   ├── aff-api/       # HTTP 层: Actix-web 路由 + JWT 中间件
│   ├── aff-notify/    # 通知: Telegram Bot + SMTP 邮件
│   └── aff-server/    # 入口: 配置加载 → DB 初始化 → 启动服务
├── frontend/          # Vue 3 + TailwindCSS + vue-i18n 前端
├── vendor/TokenPay/   # TokenPay 加密货币支付 (git submodule)
├── config/tokenpay/   # TokenPay 配置模板
└── config.toml        # 默认配置
```

## API 概览

### 前台 `/api/v1`
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/categories` | 商品分类列表 |
| GET | `/products` | 商品列表 |
| POST | `/orders` | 创建订单 |
| GET | `/orders/query?email=` | 按邮箱查询订单列表 |
| GET | `/orders/{order_no}?email=` | 查询单个订单详情 |
| POST | `/aff/register` | 注册 AFF 推广账号 |
| GET | `/aff/query?email=` | 查询佣金/等级信息 |
| GET | `/aff/tiers` | 查看等级体系 |
| GET | `/aff/logs?email=` | 查询佣金记录 |
| POST | `/aff/withdraw` | 申请提现 |
| GET | `/announcement` | 获取商城公告 |

### 管理 `/api/admin`
| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/auth/login` | 登录 |
| GET | `/dashboard` | 仪表盘统计 |
| CRUD | `/categories` | 分类管理 |
| CRUD | `/products` | 商品管理 |
| GET/POST/DELETE | `/cards` | 卡密管理 (POST `/cards/import` 批量导入) |
| GET | `/orders` | 订单管理 |
| GET/PUT | `/payment-configs` | 支付通道配置 |
| GET | `/withdrawals` | 提现审批 |
| PUT | `/withdrawals/{id}/approve` | 通过提现 |
| PUT | `/withdrawals/{id}/reject` | 拒绝提现 |
| GET/PUT | `/settings` | 系统设置 |
| POST | `/settings/test-telegram` | 测试 Telegram 通知 |
| POST | `/settings/test-email` | 测试 SMTP 邮件 |
| POST | `/upload` | 上传文件 (图片/视频) |
| GET | `/admins` | 管理员列表 |
| GET | `/aff/users` | AFF 用户列表 |
| CRUD | `/aff/tiers` | AFF 等级管理 |

## AFF 推广等级体系

系统内置 4 级推广等级，管理员可自定义增减：

| 等级 | 名称 | 返佣比例 | 升级门槛 |
|------|------|----------|----------|
| L1 | 青铜 | 5% | ¥0 (默认) |
| L2 | 白银 | 10% | 累计邀请 ≥¥100 |
| L3 | 黄金 | 15% | 累计邀请 ≥¥500 |
| L4 | 钻石 | 20% | 累计邀请 ≥¥1000 |

- 用户注册后默认 L1，邀请订单成交后自动计算佣金并检查升级
- 商品可单独设置返佣比例，优先于等级比例
- 管理员可在后台自由增删等级、调整比例和门槛
- 提现支持 USDT/USDC，可选 Tron/Polygon/Base 链

## 多语言 & 深色模式

- 前端支持简体中文、繁体中文、English 三种语言
- 首次访问自动检测浏览器语言，可在页面顶部手动切换
- 深色模式支持: 亮色 / 暗色 / 跟随系统，点击顶部图标切换
- 偏好设置持久化到 localStorage

## 数据库后端

项目通过 Cargo feature flags 支持 SQLite 和 PostgreSQL 两种数据库：

| Feature | 说明 | 编译命令 |
|---------|------|----------|
| `sqlite` (默认) | 零配置，单文件数据库 | `cargo build --release` |
| `postgres` | 需要运行中的 PostgreSQL 实例 | `cargo build --release --features postgres --no-default-features` |

- Docker 构建时通过 `DB_BACKEND` build arg 切换 (默认 `sqlite`)
- 应用代码无差异 — SeaORM 自动根据连接串协议选择驱动
- `config.toml` 中修改 `database.url` 即可切换 (需配合对应 feature 编译)

## 开发

```bash
# 后端开发 (热重载)
cargo watch -x run

# 前端开发
cd frontend && pnpm dev

# 运行测试
cargo test
```

## License

MIT
