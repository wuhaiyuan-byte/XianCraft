use crate::world::world_player::Player;

/// Holds the context for rendering a combat message.
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub attacker: &'a Player,
    pub defender: &'a Player,
    pub body_part: &'a str,
    pub weapon: &'a str,
}

/// Renders a combat message template with dynamic values.
///
/// Placeholders:
/// - $N: Attacker's name
/// - $n: Defender's name
/// - $l: Body part
/// - $w: Weapon
pub fn render(template: &str, context: &RenderContext) -> String {
    template
        .replace("$N", &context.attacker.name)
        .replace("$n", &context.defender.name)
        .replace("$l", context.body_part)
        .replace("$w", context.weapon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::world_player::Player;

    #[test]
    fn test_render_combat_message() {
        let attacker = Player::new(1, "张三".to_string());
        let defender = Player::new(2, "李四".to_string());
        let context = RenderContext {
            attacker: &attacker,
            defender: &defender,
            body_part: "胸口",
            weapon: "长剑",
        };

        let template = "$N手中$w化作一道寒光刺向$n的$l";
        let rendered = render(template, &context);

        assert_eq!(rendered, "张三手中长剑化作一道寒光刺向李四的胸口");
    }
}
