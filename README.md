# TuhuCar CLI 途虎养车

途虎养车 CLI 工具 — 车型匹配与养车知识查询，配合 AI 编程助手使用。

## 安装

### npm（推荐）

```bash
npm install -g @tuhucar/cli
```

### 一键安装

```bash
curl -fsSL https://raw.githubusercontent.com/tuhucar/cli/main/install.sh | sh
```

### Homebrew

```bash
brew install tuhucar/tap/tuhucar
```

## 快速开始

```bash
# 初始化配置
tuhucar config init

# 匹配车型
tuhucar car match "2024款朗逸1.5L自动舒适版"

# 查询养车知识
tuhucar knowledge query --car-id <car_id> "多久换机油"

# 查看命令结构（供 LLM 自省）
tuhucar car schema
tuhucar knowledge schema
```

## 命令参考

### 全局选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--format json\|markdown` | 输出格式 | `markdown` |
| `--dry-run` | 预览请求，不实际发送 | 关闭 |
| `--verbose` | 详细输出 | 关闭 |
| `--version` | 显示版本号 | - |
| `--help` | 显示帮助 | - |

### 车型匹配

```bash
# 模糊匹配车型
tuhucar car match "朗逸"

# JSON 格式输出（供程序使用）
tuhucar car match "2024款朗逸1.5L" --format json
```

### 养车知识查询

```bash
# 查询指定车型的养车知识
tuhucar knowledge query --car-id 12345 "多久换机油"

# 预览请求
tuhucar knowledge query --car-id 12345 "轮胎气压" --dry-run
```

### 配置管理

```bash
# 初始化默认配置
tuhucar config init

# 查看当前配置
tuhucar config show
```

配置文件位置：`~/.tuhucar/config.toml`

### Skill 管理

```bash
# 安装 Skill 到检测到的 AI 平台
tuhucar skill install

# 卸载 Skill
tuhucar skill uninstall
```

## AI 平台集成

TuhuCar CLI 提供 Skill 定义，支持以下 AI 编程助手：

| 平台 | 状态 |
|------|------|
| Claude Code | ✓ 支持 |
| Cursor | ✓ 支持 |
| Codex | ✓ 支持 |
| OpenCode | ✓ 支持 |
| Gemini CLI | ✓ 支持 |

安装 CLI 后运行 `tuhucar skill install`，会自动检测已安装的平台并注册 Skill。

## 项目结构

```
tuhucar/
├── crates/
│   ├── tuhucar-core/       # 公共基础（配置、HTTP、错误、类型）
│   ├── tuhucar-car/        # 车型匹配模块
│   ├── tuhucar-knowledge/  # 养车知识查询模块
│   └── tuhucar-cli/        # CLI 二进制入口
├── skills/                  # AI Skill 定义
├── npm/                     # npm 分发包
└── scripts/                 # 安装脚本
```

## 开发

```bash
# 构建
cargo build

# 运行测试
cargo test --workspace

# 运行 CLI
cargo run -p tuhucar-cli -- --help
```

## License

MIT
