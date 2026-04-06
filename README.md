# AFF Card Shop (发卡商城)

自动发卡商城系统 — Rust (Actix-web) + Vue 3 + TailwindCSS

## 功能特性

- **自动发卡**: 用户购买后自动发放卡密，邮箱查询订单
- **管理后台**: 商品/分类/卡密/订单 CRUD，批量导入补货，仪表盘统计
- **双支付通道**: 易支付 (支付宝/微信/QQ) + TokenPay (USDT/TRX 加密货币)
- **AFF 推广系统**: 邀请返佣，多级等级体系 (青铜→白银→黄金→钻石)，自动升级
- **多链提现**: 支持 USDT/USDC 在 Tron、Polygon、Base 链提现
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
| GET | `/orders/query?email=` | 按邮箱查询订单列表 |
| GET | `/orders/{order_no}?email=` | 查询单个订单详情 |
| POST | `/aff/register` | 注册 AFF 推广账号 |
| GET | `/aff/query?email=` | 查询佣金/等级信息 |
| GET | `/aff/tiers` | 查看等级体系 |
| GET | `/aff/logs?email=` | 查询佣金记录 |
| POST | `/aff/withdraw` | 申请提现 |

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
