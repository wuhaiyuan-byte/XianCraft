#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::thread::sleep;
    use std::time::Duration;
    use colored::*;

    fn display(msg: &str) {
        print!("{}", msg);
        io::stdout().flush().unwrap();
    }

    fn draw_bar(label: &str, current: u32, max: u32, color: Color, width: usize) -> String {
        let fill_width = (current as f32 / max as f32 * width as f32).round() as usize;
        let fill = "█".repeat(fill_width).color(color);
        let empty = "░".repeat(width - fill_width).on_truecolor(40, 40, 40);

        format!(
            "{:<5} {}{} {:>4}/{:<4}",
            label,
            fill,
            empty,
            current,
            max
        )
    }

    #[test]
    fn test_visual_demo() {
        println!("

");
        // Legendary Orange Title
        display(&format!("{}
", "╔══════════════════════════════════════════════╗".truecolor(100, 100, 100)));
        display(&format!("{}{}{}
", 
            "║".truecolor(100, 100, 100),
            "        仙途缥缈 - World of Warcraft Style        ".bold().truecolor(255, 128, 0),
            "║".truecolor(100, 100, 100)
        ));
        display(&format!("{}
", "╚══════════════════════════════════════════════╝".truecolor(100, 100, 100)));
        
        sleep(Duration::from_millis(600));

        // 1. Attribute Panel with Rare Blue Title
        display(&format!("
{}
", "【 角色属性面板 】".bold().truecolor(0, 112, 221)));
        display(&format!("{}
", "╔══════════════════════════════════════════════╗".truecolor(100, 100, 100)));
        display(&format!("{}{}{}
", 
            "║ ".truecolor(100, 100, 100),
            "姓名：测试剑修          境界：炼气三层     ".white(),
            "║".truecolor(100, 100, 100)
        ));
        display(&format!("{}                                              {}
", "║".truecolor(100, 100, 100), "║".truecolor(100, 100, 100)));
        // Health (Death Knight Red), Mana (Mage Blue), Energy (Rogue Yellow)
        display(&format!("{}  {}  {}
", "║".truecolor(100, 100, 100), draw_bar("生命", 120, 200, Color::TrueColor { r: 196, g: 31, b: 59 }, 20), "║".truecolor(100, 100, 100)));
        display(&format!("{}  {}  {}
", "║".truecolor(100, 100, 100), draw_bar("真元", 45, 150, Color::TrueColor { r: 105, g: 204, b: 240 }, 20), "║".truecolor(100, 100, 100)));
        display(&format!("{}  {}  {}
", "║".truecolor(100, 100, 100), draw_bar("精力", 85, 100, Color::TrueColor { r: 255, g: 245, b: 105 }, 20), "║".truecolor(100, 100, 100)));
        display(&format!("{}
", "╚══════════════════════════════════════════════╝".truecolor(100, 100, 100)));

        sleep(Duration::from_millis(800));

        // 2. Combat Sequence with Uncommon Green enemy
        display(&format!("
{}:
", "战斗开始".bold().truecolor(255, 245, 105)));
        display(&format!("{}
", "你身形如电，长剑微鸣，带起一道残影。".white()));
        sleep(Duration::from_millis(400));
        display(&format!("你对着 {} 发起猛攻，剑光在竹林间闪烁。
", "翠竹蛇".truecolor(30, 255, 0).bold()));
        sleep(Duration::from_millis(400));
        display(&format!("{}
", "翠竹蛇挣扎了几下，最终无力地倒在草丛中。".white()));
        
        sleep(Duration::from_millis(500));

        // 3. Quest Progress (Uncommon Green and White)
        display(&format!("
{} {}: {}
", "[ 任务进度 ]".truecolor(30, 255, 0).bold(), "".normal(), "清缴翠竹蛇 (8/10)".white().bold()));
        
        sleep(Duration::from_millis(1000));

        // 4. Epic Purple Achievement
        display(&format!("
{}
", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".truecolor(163, 53, 238).bold()));
        display(&format!("      {}
", "恭喜！你完成了悬赏：[清缴翠竹蛇]".truecolor(163, 53, 238).bold()));
        display(&format!("{}
", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".truecolor(163, 53, 238).bold()));
        // Legendary Orange and Rare Blue rewards
        display(&format!("获得奖励：{}  {}
", "灵贝 +200".truecolor(255, 128, 0), "修为 +500".truecolor(0, 112, 221).bold()));
        
        sleep(Duration::from_millis(600));

        // 5. Epic Purple Level Up
        display(&format!("
{}
", "【突破】".truecolor(163, 53, 238).bold()));
        display(&format!("{}
", "你感到体内真元澎湃，经脉拓宽，顺利晋升至 [炼气第四层]！".truecolor(163, 53, 238).bold()));
        
        println!("

");
    }
}