#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use colored::*;

    fn display(msg: &str) {
        print!("{}", msg);
        io::stdout().flush().unwrap();
    }

    #[test]
    fn run_welcome_demos() {
        println!("\n{}", "================================================".truecolor(255, 215, 0));
        println!("{}", "       ✨ 欢迎界面创意方案 (Welcome Demos) ✨     ".truecolor(255, 215, 0).bold());
        println!("{}\n", "================================================".truecolor(255, 215, 0));

        // --- Demo #1: [ 仙山云海 - Zen & Landscape Style ] ---
        println!("{}", "[ Demo #1: 仙山云海 ]".bold().white());
        display(&format!(
            "{}          {}  .   *  .  \n\
            {}         /  \\   {}  .  /\\  .    \n\
            {}/\\    /    \\     /  \\   \n\
            {}/  \\  /      \\   /    \\  \n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\n",
            "          /\\".truecolor(75, 175, 80),
            ".   *  .".truecolor(220, 220, 220),
            "         /  \\".truecolor(65, 165, 70),
            ".  /\\".truecolor(200, 200, 200),
            "  /\\    /    \\".truecolor(55, 155, 60),
            " /  \\  /      \\".truecolor(45, 145, 50),
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~".white(),
            "      ☯  仙 途 缥 缈  ☯      ".bold().truecolor(135, 206, 250),
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~".white(),
            "   “ 道 法 自 然 ， 心 无 旁 骛 ”   ".truecolor(180, 180, 180)
        ));

        // --- Demo #2: [ 修真之路 - Epic Journey Style ] ---
        println!("{}", "[ Demo #2: 修真之路 ]".bold().white());
        display(&format!(
            "{}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\n",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red(),
            "  ⚔️   修  仙  传  说   ⚔️  ".bold().yellow(),
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red(),
            "   踏碎凌霄，放眼万世。                ".white(),
            "   这漫漫长生路，谁主沉浮？            ".white(),
            "   >> [ 三 千 大 道 ， 唯 我 不 败 ] <<  ".bold().truecolor(255, 165, 0),
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red()
        ));

        // --- Demo #3: [ 星辰大海 - Cosmic & Mystical Style ] ---
        println!("{}", "[ Demo #3: 星辰大海 ]".bold().white());
        display(&format!(
            "{}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\n",
            "     ★          ✧          ✦     ".truecolor(150, 150, 255),
            format!("  .     {} 诸 天 万 界 {}.     .", "☯".bold().yellow(), "☯".bold().yellow()).truecolor(160, 160, 255),
            "      ✧          ✦          ★     ".truecolor(170, 170, 255),
            "  ------------------------------  ".truecolor(180, 180, 255),
            "   星光汇聚之处，即是永生之门。        ".truecolor(200, 200, 255),
            "   “ 诸 天 万 界 ， 皆 入 我 囊 ”      ".bold().cyan(),
            "  ------------------------------  ".truecolor(180, 180, 255)
        ));

        println!("{}", "请选择您心仪的编号 (1-3) 并告诉我也将其更新至配置文件。".bold().yellow());
    }
}
