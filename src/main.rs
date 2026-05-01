mod output;

fn main() {
    output::main();
}

#[allow(dead_code)]
fn logic() {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::io::{self, Write, Read};
    use std::sync::mpsc;
    use std::thread;

    const W: usize = 200;
    const H: usize = 49;

    const SCALE_X: usize = 2; 
    const SCALE_Y: usize = 2;
    const CHAR_GAP: usize = 2;
    const HOUR_OFFSET: i64 = -4; 

    const HASH: char = 35 as char;
    const SPACE: char = 32 as char;
    const NEWLINE: char = 10 as char;
    const QUESTION: char = 63 as char;
    
    let glyphs: [(char, u128); 11] = [
        ('0', 285237663201400512013884), ('1', 115552141041864745162878),
        ('2', 589226334952290506557183), ('3', 589226339435917013862012),
        ('4', 57188107787330554432524),  ('5', 1207759133990284117329532),
        ('6', 285237463075990605620796), ('7', 1204315001691522129223776),
        ('8', 285237636875394176214588), ('9', 285237663099675272963644),
        (':', 1736137656755552256),
    ];

    let s = r###"?"###;
    let mut expanded = String::new();
    for c in s.chars() {
        match c {
            QUESTION => expanded.push_str(s),
            HASH => expanded.push_str("\x23\x23\x23"),
            _ => expanded.push(c),
        }
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = expanded.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() { i += 1; }
        else {
            let mut token = String::new();
            while i < chars.len() && !chars[i].is_whitespace() {
                token.push(chars[i]);
                i += 1;
            }
            tokens.push(token);
        }
    }

    let mut lines: Vec<Vec<String>> = vec![Vec::new(); H];
    let mut current_token_idx = 0;
    for row in 0..H {
        let remaining_rows = H - row;
        let mut count = 0;
        let mut width = 0;
        while current_token_idx + count < tokens.len() {
            let t_len = tokens[current_token_idx + count].chars().count();
            let space = if count == 0 { 0 } else { 1 };
            if width + space + t_len > W { break; }
            width += space + t_len;
            count += 1;
            if (tokens.len() - (current_token_idx + count)) < (remaining_rows - 1) { break; }
        }
        if remaining_rows == 1 { count = tokens.len() - current_token_idx; }
        let count = count.max(if current_token_idx < tokens.len() { 1 } else { 0 });
        for _ in 0..count {
            if current_token_idx < tokens.len() {
                lines[row].push(tokens[current_token_idx].clone());
                current_token_idx += 1;
            }
        }
    }

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0; 1];
        while io::stdin().read_exact(&mut buffer).is_ok() {
            if buffer[0] == b'q' {
                let _ = tx.send(());
                break;
            }
        }
    });

    print!("\x1b[2J\x1b[{}25l", QUESTION);
    loop {
        if rx.try_recv().is_ok() {
            print!("\x1b[{}25h\x1b[2J\x1b[H", QUESTION);
            break;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let local_now = now + (HOUR_OFFSET * 3600);
        let h_time = (local_now / 3600 % 24).abs() as usize;
        let m_time = (local_now / 60 % 60).abs() as usize;
        let s_time = (local_now % 60).abs() as usize;
        let time_str = format!("{:02}:{:02}:{:02}", h_time, m_time, s_time);

        let g_w = 8;
        let g_h = 10;
        let scaled_g_w = g_w * SCALE_X;
        let scaled_g_h = g_h * SCALE_Y;
        let clock_w = (time_str.len() * scaled_g_w) + (time_str.len() - 1) * CHAR_GAP;
        let start_x = (W.saturating_sub(clock_w)) / 2;
        let start_y = (H.saturating_sub(scaled_g_h)) / 2;

        let mut canvas = vec![vec![false; W]; H];
        for (idx, c) in time_str.chars().enumerate() {
            if let Some(&(_, bits)) = glyphs.iter().find(|(g, _)| *g == c) {
                for py in 0..g_h {
                    for px in 0..g_w {
                        let bit_pos = (g_h - 1 - py) * g_w + (g_w - 1 - px);
                        if (bits >> bit_pos) & 1 == 1 {
                            for sy in 0..SCALE_Y {
                                for sx in 0..SCALE_X {
                                    let x = start_x + idx * (scaled_g_w + CHAR_GAP) + (px * SCALE_X) + sx;
                                    let y = start_y + (py * SCALE_Y) + sy;
                                    if x < W && y < H { canvas[y][x] = true; }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut frame = String::with_capacity(W * H * 15);
        frame.push_str("\x1b[H"); 
        for row in 0..H {
            let line_tokens = &lines[row];
            let mut line_chars = Vec::new();
            if line_tokens.is_empty() {
                line_chars = vec![SPACE; W];
            } else {
                let content_len: usize = line_tokens.iter().map(|t| t.chars().count()).sum();
                let total_spaces = W.saturating_sub(content_len);
                let gaps = if line_tokens.len() > 1 { line_tokens.len() - 1 } else { 0 };
                if gaps > 0 {
                    let mut distributed = 0;
                    for (idx, t) in line_tokens.iter().enumerate() {
                        line_chars.extend(t.chars());
                        if idx < gaps {
                            let target = (total_spaces * (idx + 1)) / gaps;
                            let to_add = target.saturating_sub(distributed);
                            for _ in 0..to_add { line_chars.push(SPACE); }
                            distributed += to_add;
                        }
                    }
                } else {
                    line_chars.extend(line_tokens[0].chars());
                    while line_chars.len() < W { line_chars.push(SPACE); }
                }
            }

            for (col, &c) in line_chars.iter().enumerate().take(W) {
                if canvas[row][col] && c != SPACE {
                    frame.push_str("\x1b[1;31m");
                    frame.push(c);
                    frame.push_str("\x1b[0m");
                } else {
                    frame.push_str("\x1b[38;5;236m");
                    frame.push(c);
                    frame.push_str("\x1b[0m");
                }
            }
            if row < H - 1 { frame.push(NEWLINE); }
        }
        print!("{}", frame);
        io::stdout().flush().unwrap();
        thread::sleep(std::time::Duration::from_millis(200));
    }
}