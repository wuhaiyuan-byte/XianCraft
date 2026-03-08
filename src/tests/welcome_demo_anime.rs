#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    fn display(msg: &str) {
        print!("{}", msg);
        io::stdout().flush().unwrap();
    }

    #[test]
    fn run_anime_xianxia_demo() {
        println!("\n\n\x1b[1;38;5;220m====================================================\x1b[0m");
        println!("\x1b[1;38;5;218m        ✨ 二次元修仙 - 欢迎界面风格预览 ✨         \x1b[0m");
        println!("\x1b[1;38;5;220m====================================================\x1b[0m");

        // --- Demo #4: 剑灵少女 (Moe Sword Spirit) ---
        println!("\n\x1b[1;38;5;183mDemo #4: [ 剑灵少女 - Moe Sword Spirit Style ]\x1b[0m");
        let demo4 = format!(
            "\x1b[38;5;218m          /\\_/\\  \x1b[38;5;183m ⚔️  ✨  ⚔️\x1b[0m\n\
             \x1b[38;5;218m         ( ◕‿◕✿)  \x1b[38;5;255m< [ 灵剑：绯樱 ]\x1b[0m\n\
             \x1b[38;5;218m          (  つつ \x1b[38;5;183m🌸 🌸 🌸\x1b[0m\n\
             \x1b[38;5;218m           |  |  |\x1b[0m\n\
             \x1b[38;5;218m           V--V  \x1b[0m\n\
             \x1b[1;38;5;218m  “欧尼酱！快握紧这把灵剑，一起踏上登仙之路吧~”\x1b[0m\n"
        );
        display(&demo4);

        // --- Demo #5: 异世界转生 (Isekai Rebirth) ---
        println!("\n\x1b[1;38;5;45mDemo #5: [ 异世界转生 - Isekai Rebirth Style ]\x1b[0m");
        let demo5 = format!(
            "\x1b[38;5;45m        ✧─────────【 召唤阵载入中 】─────────✧\x1b[0m\n\
             \x1b[38;5;171m             ✡       ✧       ☯             \x1b[0m\n\
             \x1b[38;5;45m          ✦      \x1b[38;5;255mRe:Birth\x1b[38;5;45m      ✦          \x1b[0m\n\
             \x1b[38;5;171m             ☯       ✧       ✡             \x1b[0m\n\
             \x1b[38;5;45m        ✧──────────────────────────────────✧\x1b[0m\n\
             \x1b[1;38;5;226m  “关于我转生到修仙世界变成废材剑修这档事”\x1b[0m\n"
        );
        display(&demo5);

        // --- Demo #6: 赛博仙途 (Cyber-Xianxia) ---
        println!("\n\x1b[1;38;5;198mDemo #6: [ 赛博仙途 - Cyber-Xianxia Style ]\x1b[0m");
        let demo6 = format!(
            "\x1b[38;5;46m  [ SYSTEM ] >>> \x1b[38;5;198mDATA_STREAM_CONNECTED\x1b[0m\n\
             \x1b[38;5;46m  0101 \x1b[38;5;255m╔═══════════════════════╗\x1b[38;5;46m 1010\x1b[0m\n\
             \x1b[38;5;46m  1100 \x1b[1;38;5;198m║  大 衍 神 诀 . e x e  ║\x1b[38;5;46m 0011\x1b[0m\n\
             \x1b[38;5;46m  0010 \x1b[38;5;255m╚═══════════════════════╝\x1b[38;5;46m 0101\x1b[0m\n\
             \x1b[38;5;198m  [ WARNING ] : \x1b[38;5;255m检测到真元溢出，系统强制重写中...\x1b[0m\n\
             \x1b[1;38;5;46m  “警告：检测到真元溢出，系统开始载入[大衍神诀.exe]...”\x1b[0m\n"
        );
        display(&demo6);

        println!("\x1b[1;38;5;220m====================================================\x1b[0m\n");
    }
}