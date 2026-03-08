#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::thread::sleep;
    use std::time::Duration;

    // ANSI Color Constants (256-color palette)
    const RESET: &str = "\x1b[0m";
    const BORDER: &str = "\x1b[38;5;244m";      // Grey border
    const BG_EMPTY: &str = "\x1b[48;5;236m";    // Dark Grey background for bars
    const HP_RED: &str = "\x1b[38;5;160m";      // Deep Red
    const QI_CYAN: &str = "\x1b[38;5;33m";      // Deep Cyan
    const STAMINA_GOLD: &str = "\x1b[38;5;214m"; // Orange/Gold
    const DIM_WHITE: &str = "\x1b[38;5;250m";   // Dim White for text
    const VIBRANT_GREEN: &str = "\x1b[1;38;5;46m"; // Bold Vibrant Green
    const BOLD_GOLD: &str = "\x1b[1;38;5;220m";   // Bold Achievement Gold

    fn display(msg: &str) {
        print!("{}", msg);
        io::stdout().flush().unwrap();
    }

    fn draw_bar(label: &str, current: u32, max: u32, color: &str, width: usize) -> String {
        let fill_width = (current as f32 / max as f32 * width as f32).round() as usize;
        let mut bar = format!("{:<5} ", label);
        
        // Solid part
        bar.push_str(color);
        for _ in 0..fill_width { bar.push('█'); }
        bar.push_str(RESET);

        // Empty part with consistent grey background
        bar.push_str(BG_EMPTY);
        for _ in fill_width..width { bar.push('░'); }
        bar.push_str(RESET);

        bar.push_str(&format!(" {:>4}/{:<4}", current, max));
        bar
    }

    #[test]
    fn test_visual_demo() {
        println!("\n\n");
        display(&format!("{}╔══════════════════════════════════════════════╗{}\n", BORDER, RESET));
        display(&format!("{}║        仙途缥缈 - 视觉风格预览 (Refined)     ║{}\n", BOLD_GOLD, RESET));
        display(&format!("{}╚══════════════════════════════════════════════╝{}\n", BORDER, RESET));
        
        sleep(Duration::from_millis(600));

        // 1. Refined Attribute Panel
        display(&format!("\n{}【 角色属性面板 】{}\n", BOLD_GOLD, RESET));
        display(&format!("{}╔══════════════════════════════════════════════╗{}\n", BORDER, RESET));
        display(&format!("{}║{} 姓名：测试剑修          境界：炼气三层     {}║\n", BORDER, DIM_WHITE, BORDER));
        display(&format!("{}║                                              ║{}\n", BORDER, RESET));
        display(&format!("{}║  {}  {}║\n", BORDER, draw_bar("生命", 120, 200, HP_RED, 20), BORDER));
        display(&format!("{}║  {}  {}║\n", BORDER, draw_bar("真元", 45, 150, QI_CYAN, 20), BORDER));
        display(&format!("{}║  {}  {}║\n", BORDER, draw_bar("精力", 85, 100, STAMINA_GOLD, 20), BORDER));
        display(&format!("{}╚══════════════════════════════════════════════╝{}\n", BORDER, RESET));

        sleep(Duration::from_millis(800));

        // 2. Combat Sequence
        display(&format!("\n{}战斗开始：{}\n", BOLD_GOLD, RESET));
        display(&format!("{}你身形如电，长剑微鸣，带起一道残影。{}\n", DIM_WHITE, RESET));
        sleep(Duration::from_millis(400));
        display(&format!("{}你对着 {}翠竹蛇{} 发起猛攻，剑光在竹林间闪烁。{}\n", DIM_WHITE, VIBRANT_GREEN, DIM_WHITE, RESET));
        sleep(Duration::from_millis(400));
        display(&format!("{}翠竹蛇挣扎了几下，最终无力地倒在草丛中。{}\n", DIM_WHITE, RESET));
        
        sleep(Duration::from_millis(500));

        // 3. Quest Progress
        display(&format!("\n{} [ 任务进度 ] {}: {}清缴翠竹蛇 (8/10) {}\n", VIBRANT_GREEN, RESET, BOLD_GOLD, RESET));
        
        sleep(Duration::from_millis(1000));

        // 4. Critical Achievement / Reward
        display(&format!("\n{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}\n", BOLD_GOLD, RESET));
        display(&format!("{}      恭喜！你完成了悬赏：[清缴翠竹蛇]        {}\n", BOLD_GOLD, RESET));
        display(&format!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}\n", BOLD_GOLD, RESET));
        display(&format!("{}获得奖励：{}灵贝 +200  {}修为 +500{}\n", DIM_WHITE, STAMINA_GOLD, VIBRANT_GREEN, RESET));
        
        sleep(Duration::from_millis(600));

        // 5. Level Up / Realm Breakthrough
        display(&format!("\n{}【突破】{}\n", VIBRANT_GREEN, RESET));
        display(&format!("{}你感到体内真元澎湃，经脉拓宽，顺利晋升至 [炼气第四层]！{}\n", VIBRANT_GREEN, RESET));
        
        println!("\n");
    }
}