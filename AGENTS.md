# AGENTS.md - 项目开发指南

## 项目概述

这是一个 Rust MUD (Multi-User Dungeon) 游戏服务器，采用 xianxia（修仙）主题，使用 axum + WebSocket + React 构建。

## 技术栈

- **后端**: Rust + axum + tokio
- **前端**: React + Vite
- **通信**: WebSocket (JSON 格式)
- **数据存储**: 内存 (运行时加载 JSON 数据)

## 项目结构

```
src/
├── lib.rs              # 主服务器逻辑，WebSocket 处理，命令路由
├── command.rs          # 命令解析 (parse 函数)
├── combat.rs           # 战斗引擎核心 (伤害计算、暴击判定)
├── game_loop.rs       # 游戏循环 (心跳恢复、战斗tick)
├── ui.rs              # 界面显示函数 (欢迎消息、房间描述、who列表)
├── npc.rs             # NPC 定义
├── main.rs            # 程序入口
├── world/                         # 游戏世界数据
│   ├── world_state.rs             # 游戏世界状态 (动态数据)
│   ├── world_player.rs            # 玩家数据结构
│   ├── world_loader.rs            # 数据加载
│   ├── world_room.rs              # 房间定义
│   ├── world_skill.rs             # 技能数据结构
│   ├── world_zone.rs              # 区域定义
│   ├── world_text.rs              # 文本渲染
│   ├── world_event.rs             # 事件系统
│   └── mod.rs
├── commands/                      # 命令模块 (按功能分组)
│   ├── mod.rs                     # 模块导出
│   ├── help.rs                    # help 命令
│   ├── look.rs                    # look 命令
│   ├── who.rs                     # who 命令
│   ├── player.rs                  # score/inventory/rest/work 命令
│   ├── battle.rs                  # attack/kill/cast 命令
│   ├── movement.rs                # go 命令
│   ├── interaction.rs             # talk/get 命令
│   └── quest.rs                   # quest/accept 命令
├── world_model.rs      # 静态数据结构 (Quest, Item, Skill 等)
└── tests/              # 单元测试
```

## 核心架构

### 1. 数据分离

- **静态数据** (`WorldState.static_data`): 房间、NPC 模板、物品模板、技能模板、任务配置 (从 JSON 加载)
- **动态数据** (`WorldState.dynamic_data`): 在线玩家位置、NPC 实例、房间物品

### 2. 状态管理

```rust
struct AppState {
    world_state: WorldState,                          // 游戏世界
    player_sessions: Mutex<HashMap<u64, PlayerSession>>, // 在线玩家会话
}
```

### 3. 消息协议

客户端发送:
```json
{ "type": "Login", "user_id": "用户名" }
{ "type": "Command", "command": "look" }
```

服务器返回:
```json
{ "type": "Description", "payload": "..." }
{ "type": "Info", "payload": "..." }
{ "type": "Error", "payload": "..." }
```

## 开发原则

### 1. 锁的使用规则 (关键!)

**避免死锁的核心原则:**

1. **禁止嵌套锁**: 不要在持有 `player_sessions` 锁的同时再获取其他锁
2. **锁的顺序**: 如果必须获取多个锁，必须按固定顺序
3. **复制数据而非引用**: 在锁内复制需要的数据，释放锁后再处理

**正确示例:**
```rust
// 错误: 嵌套锁 - 会死锁!
let session = sessions.lock().unwrap();
// ... 一些操作 ...
let data = world.dynamic_data.lock().unwrap(); // 危险!

// 正确: 先复制数据，释放锁
let player_ids: Vec<u64> = {
    let data = world.dynamic_data.lock().unwrap();
    data.players.keys().cloned().collect()
}; // 锁已释放

let names: Vec<String> = {
    let sessions = state.player_sessions.lock().unwrap();
    player_ids.iter().filter_map(|id| sessions.get(id)).collect()
};
```

**补充：已持有 session_lock 时获取其他数据**

如果已经持有 `session_lock`，需要获取其他数据源（如 `dynamic_data`），必须：
1. 先提取需要的简单数据（如 player_prefix、room_id）
2. 释放 `session_lock`（使用 `drop(session_lock)`）
3. 再获取其他锁

### 2. 玩家状态同步

玩家名称和位置信息需要同步到两个地方:

1. **player_sessions**: 用于 `who` 命令显示所有在线玩家
2. **world.dynamic_data.players**: 用于 `look` 命令显示同房间玩家

登录时设置:
```rust
session.user_id = Some(user_id.clone());
state.world_state.update_player_name(player_id, user_id);
```

移动时更新:
```rust
world.move_player_to_room(player_id, &next_room_id, session.user_id.clone());
```

### 3. 命令处理流程

1. WebSocket 接收消息 → `parse()` 解析命令
2. `handle_command()` 处理命令
3. 获取 session 锁 → 执行逻辑 → 发送响应

### 4. 添加新命令

1. 在 `command.rs` 的 `Command` enum 添加变体
2. 在 `command.rs` 的 `parse()` 添加解析逻辑
3. 在对应的 `commands/` 目录下创建或更新命令处理函数
4. 在 `lib.rs` 的 `handle_command()` 调用 commands 模块
5. 在 `commands/help.rs` 添加命令说明

**commands 目录结构:**
- 按功能分组创建文件 (如 battle.rs, movement.rs)
- 在 `commands/mod.rs` 导出
- 函数签名设计为接收必要参数，返回 `ServerMessage` 或 `Option<ServerMessage>`

## 技能系统

技能定义在 `data/skills.json`，包含:
- **物理技能**: sword_1 (太宗剑法)
- **法术技能**: thunder_1 (掌心雷)
- **治疗技能**: heal_1 (灵气疗伤)

每个技能有:
- combo连击系统 (3个招式循环)
- 属性加成 (str/dex/int)
- 冷却时间

## 任务系统

- **类型**: 击杀任务 (kill)、对话任务 (talk)、移动任务 (move)
- **状态跟踪**: `PlayerQuestStatus`
- **奖励**: 灵贝、修为、潜能

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name
```

## 常用命令

```bash
# 启动服务器
cargo run

# 运行测试
cargo test

# 构建
cargo build

# 检查代码
cargo check
```

## 代码规范

- 使用 `colored` crate 实现彩色输出
- 使用 `tracing` 进行日志记录
- 遵循 Rust 所有权规则，避免克隆过多
- 保持函数简洁，单一职责
- 命令处理逻辑应放在 `commands/` 目录下
