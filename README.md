# AFF Card Shop (发卡商城)

自动发卡商城系统 — Rust (Actix-web) + Vue 3 + TailwindCSS

## 功能特性

- **自动发卡**: 用户购买后自动发放卡密，邮箱查询订单
- **管理后台**: 商品/分类/卡密/订单 CRUD，批量导入补货，仪表盘统计
- **双支付通道**: 易支付 (支付宝/微信/QQ) + TokenPay (USDT/TRX 加密货币)
- **AFF 推广系统**: 邀请返佣，邮箱查询余额，多链 USDT/USDC 提现
- **支付后触发**: 支持 Webhook 回调和 Shell 命令执行
- **模块化架构**: Cargo Workspace 6 个 crate，清晰分层

## 快速开始

### 环境要求
- Rust 1.75+
- Node.js 18+ & pnpm

### 启动

```bash
# 1. 构建前端
cd frontend && pnpm install && pnpm run build && cd ..

# 2. 运行服务器
cargo run
```

服务器启动在 `http://127.0.0.1:8080`

- 前台商城: `http://127.0.0.1:8080/`
- 管理后台: `http://127.0.0.1:8080/admin`
- 默认管理员: `admin` / `admin123`

### 配置

复制并修改配置文件:
```bash
cp config.toml config.local.toml
```

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite:./aff_shop.db?mode=rwc"

[jwt]
secret = "your-jwt-secret-change-this"
expiration_hours = 24

[default_admin]
username = "admin"
password = "admin123"
```

## 支付通道配置

### 易支付 (pay.myzfw.com)
在管理后台 → 支付通道 → 易支付，填写:
- 商户 ID (pid)
- 商户私钥 (RSA)
- 平台公钥 (RSA)
- API 地址 (默认 https://pay.myzfw.com)

### TokenPay
1. 部署 [TokenPay](https://github.com/LightCountry/TokenPay) 服务
2. 在管理后台 → 支付通道 → TokenPay，填写:
   - API 地址 (如 http://127.0.0.1:5000)
   - 通知密钥

## 项目结构

```
AFF/
├── crates/
│   ├── aff-common/    # 共享: 配置、错误、类型、加密、ID 生成
│   ├── aff-entity/    # 数据层: SeaORM Entity + Migration + DTO
│   ├── aff-payment/   # 支付: PaymentProvider trait + 易支付/TokenPay
│   ├── aff-core/      # 业务逻辑: 订单、卡密、AFF、提现等 Service
│   ├── aff-api/       # HTTP 层: Actix-web 路由 + JWT 中间件
│   └── aff-server/    # 入口: 配置加载 → DB 初始化 → 启动服务
├── frontend/          # Vue 3 + TailwindCSS 前端
└── config.toml        # 默认配置
```

## API 概览

### 前台 `/api/v1`
| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/categories` | 商品分类列表 |
| GET | `/products` | 商品列表 |
| POST | `/orders` | 创建订单 |
| GET | `/orders/{no}?email=` | 查询订单 |
| POST | `/aff/register` | 注册 AFF |
| GET | `/aff/query?email=` | 查询佣金 |
| POST | `/aff/withdraw` | 申请提现 |

### 管理 `/api/admin`
| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/auth/login` | 登录 |
| GET | `/dashboard` | 仪表盘 |
| CRUD | `/categories` | 分类管理 |
| CRUD | `/products` | 商品管理 |
| CRUD | `/cards` | 卡密管理 |
| GET | `/orders` | 订单管理 |
| PUT | `/payment-configs/{ch}` | 支付配置 |
| GET/PUT | `/withdrawals/{id}/*` | 提现审批 |
| GET/PUT | `/settings` | 系统设置 |

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
