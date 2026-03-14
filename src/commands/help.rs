use crate::ServerMessage;
use colored::*;

pub fn handle_help() -> ServerMessage {
    let help_text = concat!(
        "----【 可用指令 (Commands) 】----\n",
        "\n  【通用】\n",
        "  look              - 查看当前环境。\n",
        "  score/status      - 查看你的角色状态。\n",
        "  inventory/i       - 查看你的背包。\n",
        "  say <内容>        - 对房间里的所有人说话。\n",
        "\n  【移动】\n",
        "  go <方向>         - 向指定方向移动 (north, south, east, west... 或 n, s, e, w...)\n",
        "\n  【互动】\n",
        "  talk <目标>       - 与NPC对话。\n",
        "  attack <目标>     - 攻击一个目标。\n",
        "  get/take <物品>   - 从地上捡起物品。\n",
        "\n  【任务】\n",
        "  quest/qs          - 查看当前任务状态。\n",
        "  accept <任务ID>   - 从告示牌等处接受任务。\n",
        "\n  【其它】\n",
        "  rest              - 原地休息以恢复体力。\n",
        "  work              - 在特定地点劳动以赚取奖励。\n",
        "  who               - 查看当前在线的玩家。"
    );
    ServerMessage::Info {
        payload: help_text.to_string(),
    }
}
