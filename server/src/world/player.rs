use crate::world::combat::Combatant;
use crate::world::entity::Entity;
use crate::ServerMessage;
use crate::world::player_state::PlayerState;
use tokio::sync::mpsc;

/// Player 结构体代表了一个通过 WebSocket 连接到服务器的真实玩家。
#[derive(Debug)]
pub struct Player {
    pub id: usize,
    pub username: String, // Store username for easier access
    pub state: PlayerState,
    pub sender: mpsc::Sender<ServerMessage>,
}

impl Player {
    /// 向该玩家的客户端发送一条服务器消息。
    /// 这是一个统一的发送点，可以处理所有类型的消息。
    pub fn send_message(&self, message: ServerMessage) {
        // try_send 是非阻塞的。如果因为任何原因（例如玩家网络卡顿，缓冲区满了）
        // 导致消息无法立即发送，它会直接返回一个错误，而不是阻塞整个服务器。
        // 在我们的游戏循环中，这是一个安全的选择。
        let _ = self.sender.try_send(message);
    }
}

// --- Trait Implementations ---

impl Combatant for Player {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_name(&self) -> &str {
        &self.username
    }

    fn get_state(&self) -> &PlayerState {
        &self.state
    }

    fn get_mut_state(&mut self) -> &mut PlayerState {
        &mut self.state
    }

    /// 对于真实玩家，我们将战斗消息封装成 GameMessage 并通过 WebSocket 发送。
    fn send_combat_message(&self, message: String) {
        self.send_message(ServerMessage::GameMessage { content: message });
    }
}

impl Entity for Player {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_entity_type(&self) -> &'static str {
        "Player"
    }

    /// 这是实现从通用 Entity 到具体 Player 类型转换的关键。
    /// 当我们有一个 `Box<dyn Entity>` 时，可以调用这个方法来安全地获取 `&Player`。
    fn as_player(&self) -> Option<&Player> {
        Some(self)
    }
}

// 这两个 AsRef/AsMut 实现是关键的“粘合剂”，
// 它告诉 Rust 如何将一个 Entity（Player）“看作”一个 Combatant。
impl AsRef<dyn Combatant> for Player {
    fn as_ref(&self) -> &(dyn Combatant + 'static) {
        self
    }
}

impl AsMut<dyn Combatant> for Player {
    fn as_mut(&mut self) -> &mut (dyn Combatant + 'static) {
        self
    }
}
