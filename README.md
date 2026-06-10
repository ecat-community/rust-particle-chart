# Particle Chart

Tauri + Leptos 粒子计数器 256 通道数据可视化应用。从串口读取粒子计数器数据，解析 `#RAWD` 协议字段（两组 256 通道 hex 数据），用 ECharts 柱状图上下分屏实时展示。

## 协议交互

- protocol.md 协议格式，主要是 `#RAWD` 字段为数据字段
- data-example.txt 真实数据例子

客户端需先发送命令触发设备：

1. 发送 `27\r\n` 启动测量，设备每 10 秒自动广播一组完整协议数据
2. 发送 `\x1B\r\n` (ESC) 停止广播

## 功能需求

### 数据展示

- 解析 `#RAWD` 字段，提取两组 256 通道 hex 数据
- 上下分屏显示两个柱状图，分别对应两组通道数据
- 只显示最新一帧数据，每次收到新数据替换上一帧
- Y 轴自动缩放，两个图表共享统一最大值
- X 轴显示 0-255 通道编号，每 32 个显示一个标签
- 支持鼠标 hover 显示数值（tooltip）、点击交互

### 串口通信

- 数据从串口获取，用户可配置串口参数（端口名、波特率、数据位、停止位、校验位）
- 使用 tokio-serial 异步 I/O（async read/write），禁止 polling/sleep 轮询
- 故障自恢复：串口断开或通信失败后，自动按指数退避重试（1s → 2s → 4s → 8s → 16s → 30s max），重连成功后自动重发 `27\r\n` 恢复数据流
- 恢复过程对上层调用者透明，前端仅通过事件感知连接状态变化（connected / reconnecting / disconnected）

## 技术栈

- **后端**: Tauri (Rust) + tokio-serial
- **前端**: Leptos + charming (Apache ECharts Rust 封装)
- **图表**: ECharts Canvas 渲染，内置 tooltip / resize / 动画过渡

## 项目结构

```
particle_chart/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri 入口
│   │   ├── serial/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs       # SerialManager: async 读写 + 故障自恢复
│   │   │   └── config.rs        # 串口配置结构体
│   │   ├── protocol/
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs        # 协议状态机解析器
│   │   │   └── rawd.rs          # #RAWD 数据结构与解析
│   │   └── commands.rs          # Tauri commands
│   └── Cargo.toml
├── src/                          # Leptos 前端
│   ├── app.rs                   # 主应用
│   ├── components/
│   │   ├── serial_config.rs     # 串口配置表单
│   │   └── channel_chart.rs     # charming + Leptos 图表组件
│   └── lib.rs
├── tests/
│   └── integration/
│       └── serial_test.rs       # socat 集成测试
├── .github/workflows/ci.yml    # CI 流水线
├── clippy.toml
├── rustfmt.toml
├── deny.toml
└── Cargo.toml                   # workspace root
```

## 质量门禁

- `cargo fmt --check` — 格式检查
- `cargo clippy -- -D warnings` — lint 严格模式
- `cargo test` — 单元测试
- `cargo test --test integration` — 集成测试
- `cargo deny check` — 依赖安全/许可证检查

### 单元测试

| 模块 | 测试内容 |
|------|---------|
| `protocol/parser` | 状态机解析各种协议字段，用 `data-example.txt` 作 fixture |
| `protocol/rawd` | #RAWD 两组 256 hex 解析，边界输入 |
| `serial/config` | 串口配置参数验证 |
| `commands` | Tauri command 参数校验，Mock SerialManager |

### 集成测试

使用 socat 创建虚拟串口对，用 Rust + tokio-serial 模拟设备端：

1. socat 创建 PTY 对（如 /dev/ttys001 ↔ /dev/ttys002）
2. 模拟设备端：打开 PTY slave，等待接收 `27\r\n` 后循环发送 `data-example.txt` 内容
3. 验证：正确发送启动命令、正确解析 #RAWD 数据、串口断开后自动重连、正确发送停止命令
