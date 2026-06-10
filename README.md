# Particle Chart

Tauri + Leptos 粒子计数器 256 通道数据可视化桌面应用。

从串口读取粒子计数器数据，解析 `#RAWD` 协议字段（两组 256 通道 hex 数据），用 ECharts 柱状图上下分屏实时展示。

## 功能

- 双通道 256 通道柱状图实时展示，Y 轴自动缩放
- 串口参数可配置（端口、波特率、数据位、停止位、校验位）
- 故障自恢复：断开后指数退避重连（1s → 2s → ... → 30s）
- 可收起侧边栏，图表自动适配容器尺寸
- 窗口最小尺寸 800×500，内容等比例 flex 布局
- Tab 切换（Dashboard / Settings）

## 协议

客户端发送命令控制设备：

1. `27\r\n` — 启动测量，设备每 ~10 秒广播一组数据
2. `\x1B\r\n` (ESC) — 停止广播

数据格式为 `#RAWD` 字段，包含两组 256 通道 hex 数据。协议文档和示例数据见 `docs/`。

## 技术栈

- **后端**: Tauri 2 (Rust) + tokio-serial
- **前端**: Leptos 0.6 (WASM) + charming (ECharts Rust 绑定)
- **构建**: Trunk (WASM 打包) + cargo-tauri
- **CI**: GitHub Actions — lint + test + Windows 构建

## 项目结构

```
particle_chart/
├── src/                              # Leptos 前端 (WASM)
│   ├── app.rs                        # 主应用布局
│   ├── style.css                     # 全局样式
│   ├── tauri_bridge.rs               # Tauri invoke/event 桥接
│   └── components/
│       ├── channel_chart.rs          # ECharts 图表 + ResizeObserver
│       ├── serial_config.rs          # 串口配置表单
│       ├── top_nav.rs                # 顶部导航栏
│       └── settings_view.rs          # 设置页
├── src-tauri/                        # Tauri 后端 (Rust)
│   ├── src/
│   │   ├── main.rs                   # 入口
│   │   ├── lib.rs                    # Tauri 插件注册
│   │   ├── commands.rs               # Tauri commands
│   │   ├── serial/
│   │   │   ├── manager.rs            # 异步串口读写 + 自动重连
│   │   │   └── config.rs             # 串口配置
│   │   ├── protocol/
│   │   │   ├── parser.rs             # 协议状态机
│   │   │   └── rawd.rs              # #RAWD 解析
│   │   └── bin/
│       └── simulated_device.rs       # 模拟设备（用于测试）
│   ├── serialport-patch/             # serialport crate 本地补丁（PTY 支持）
│   ├── tauri.conf.json
│   └── Cargo.toml
├── docs/                             # 文档（不提交 git）
├── .github/workflows/ci.yml          # CI
├── Trunk.toml
└── Cargo.toml
```

## 开发

```bash
# 前端构建
trunk build

# 开发模式（需要 cargo tauri dev）
cd src-tauri && cargo tauri dev

# 模拟设备测试（需要 socat）
socat -d -d pty,raw,echo=0,link=/tmp/ttyV0 pty,raw,echo=0,link=/tmp/ttyV1
cargo run --bin simulated-device /tmp/ttyV0
```

## CI / 构建

| 触发条件 | 动作 |
|----------|------|
| push/PR to main | lint + clippy + test |
| push/PR to main | 构建 Windows AMD64 产物 |
| tag `v*` | 构建 + 创建 GitHub Release（.exe / .msi） |

Windows 构建使用 `windows-latest` runner，产物为 NSIS 安装包和 MSI。

### serialport 补丁

`serialport` crate 在 macOS PTY 上 `IOSSIOSPEED` ioctl 返回 `ENOTTY`，导致 `tokio-serial` 无法用于 socat 虚拟串口测试。`src-tauri/serialport-patch/` 对此做了补丁：忽略 ENOTTY 错误。通过 `[patch.crates-io]` 引用，不影响 Windows 构建。
