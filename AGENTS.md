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
├── npc.rs              # NPC 定义
├── world/
│   ├── world_state.rs  # 游戏世界状态 (动态数据)
│   ├── player.rs       # 玩家数据结构
│   ├── loader.rs       # 数据加载
│   ├── room.rs         # 房间定义
│   └── ...
├── world_model.rs      # 静态数据结构 (Quest, Item 等)
└── tests/              # 单元测试
```

## 核心架构

### 1. 数据分离

- **静态数据** (`WorldState.static_data`): 房间、NPC 模板、物品模板、任务配置 (从 JSON 加载)
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

### 锁的使用规则 (关键!)

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

Look/Go 命令需要用这个模式。

### 4. 玩家状态同步

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

### 5. 命令处理流程

1. WebSocket 接收消息 → `parse()` 解析命令
2. `handle_command()` 处理命令
3. 获取 session 锁 → 执行逻辑 → 发送响应

### 6. 添加新命令

1. 在 `command.rs` 的 `Command` enum 添加变体
2. 在 `command.rs` 的 `parse()` 添加解析逻辑
3. 在 `lib.rs` 的 `handle_command()` 添加处理逻辑
4. 在 `lib.rs` 的帮助文本中添加命令说明

### 7. 测试

```bash
cargo test
```

## 常用命令

```bash
# 启动服务器
cargo run

# 运行测试
cargo test

# 构建
cargo build
```

## 代码规范

- 使用 `colored` crate 实现彩色输出
- 使用 `tracing` 进行日志记录
- 遵循 Rust 所有权规则，避免克隆过多
- 保持函数简洁，单一职责
