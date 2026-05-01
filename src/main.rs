mod output;

fn main() {
    output::main();
}

#[allow(dead_code)]
fn logic() {
    use std::time::{SystemTime, UNIX_EPOCH, Instant, Duration};
    use std::io::{self, Write, Read};
    use std::sync::mpsc;
    use std::thread;
    use std::env;

    #[cfg(unix)]
    mod os_impl {
        use std::io;
        use std::os::unix::io::AsRawFd;

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct Termios {
            pub c_iflag: u32, pub c_oflag: u32, pub c_cflag: u32, pub c_lflag: u32,
            pub c_line: u8, pub c_cc: [u8; 32], pub c_ispeed: u32, pub c_ospeed: u32,
        }

        unsafe extern "C" {
            fn tcgetattr(fd: i32, termios_p: *mut Termios) -> i32;
            fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const Termios) -> i32;
        }

        const TCSANOW: i32 = 0;
        const ICANON: u32 = 0o0000002;
        const ECHO: u32 = 0o0000010;

        pub struct RawMode { pub original: Termios, pub fd: i32 }

        impl RawMode {
            pub fn enable() -> Self {
                let fd = io::stdin().as_raw_fd();
                let mut termios = unsafe { std::mem::zeroed::<Termios>() };
                unsafe { tcgetattr(fd, &mut termios) };
                let original = termios;
                termios.c_lflag &= !(ICANON | ECHO);
                unsafe { tcsetattr(fd, TCSANOW, &termios) };
                Self { original, fd }
            }
        }
        impl Drop for RawMode {
            fn drop(&mut self) { unsafe { tcsetattr(self.fd, TCSANOW, &self.original) }; }
        }
    }

    #[cfg(windows)]
    mod os_impl {
        use std::io;
        use std::os::windows::io::AsRawHandle;

        type HANDLE = *mut std::ffi::c_void;
        type DWORD = u32;
        const ENABLE_ECHO_INPUT: DWORD = 0x0004;
        const ENABLE_LINE_INPUT: DWORD = 0x0002;

        unsafe extern "system" {
            fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut DWORD) -> i32;
            fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> i32;
        }

        pub struct RawMode { pub original: DWORD, pub handle: HANDLE }

        impl RawMode {
            pub fn enable() -> Self {
                let handle = io::stdin().as_raw_handle() as HANDLE;
                let mut mode: DWORD = 0;
                unsafe { GetConsoleMode(handle, &mut mode) };
                let original = mode;
                let raw = mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT);
                unsafe { SetConsoleMode(handle, raw) };
                Self { original, handle }
            }
        }
        impl Drop for RawMode {
            fn drop(&mut self) { unsafe { SetConsoleMode(self.handle, self.original) }; }
        }
    }

    const W: usize = 250;
    const H: usize = 66;
    const SCALE_X: usize = 3; 
    const SCALE_Y: usize = 3;
    const CHAR_GAP: usize = 2;
    const HOUR_OFFSET: i64 = -4; 

    const HASH: char = 35 as char;
    const SPACE: char = 32 as char;
    const NEWLINE: char = 10 as char;
    const QUESTION: char = 63 as char;
    const BACKTICK: char = 96 as char;
    
    let glyphs: [(char, u128); 11] = [
        ('0', 285237663201400512013884), ('1', 115552141041864745162878),
        ('2', 589226334952290506557183), ('3', 589226339435917013862012),
        ('4', 57188107787330554432524),  ('5', 1207759133990284117329532),
        ('6', 285237463075990605620796), ('7', 1204315001691522129223776),
        ('8', 285237636875394176214588), ('9', 285237663099675272963644),
        (':', 1736137656755552256),
    ];

    let mut is_timer = false;
    let mut duration_secs: i64 = 0;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let target = if args[1].contains(':') { &args[1] } else if args.len() > 2 { &args[2] } else { "" };
        let parts: Vec<&str> = target.split(':').collect();
        if parts.len() == 3 {
            is_timer = true;
            let h: i64 = parts[0].parse().unwrap_or(0);
            let m: i64 = parts[1].parse().unwrap_or(0);
            let s: i64 = parts[2].parse().unwrap_or(0);
            duration_secs = h * 3600 + m * 60 + s;
        }
    }

    let s = r###"?"###;
    let mut expanded = String::new();
    for c in s.chars() {
        match c {
            QUESTION => expanded.push_str(s),
            BACKTICK => expanded.push_str("\x23["),
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

    let _raw_mode = os_impl::RawMode::enable();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0; 1];
        while io::stdin().read_exact(&mut buffer).is_ok() {
            let _ = tx.send(buffer[0]);
        }
    });

    let mut paused = false;
    let mut last_display_time = String::new();
    let mut pause_start = Instant::now();
    let mut total_paused_nanos = 0u128;
    let start_instant = Instant::now();

    print!("\x1b[{}25l\x1b[2J", QUESTION);
    loop {
        while let Ok(key) = rx.try_recv() {
            match key {
                b'q' => {
                    print!("\x1b[{}25h\x1b[2J\x1b[H", QUESTION);
                    io::stdout().flush().unwrap();
                    return;
                }
                b' ' => {
                    paused = !paused;
                    if paused {
                        pause_start = Instant::now();
                    } else {
                        total_paused_nanos += pause_start.elapsed().as_nanos();
                    }
                }
                _ => {}
            }
        }

        let time_str = if paused {
            last_display_time.clone()
        } else if is_timer {
            let active_elapsed = start_instant.elapsed().as_nanos() - total_paused_nanos;
            let remaining = (duration_secs - (active_elapsed / 1_000_000_000) as i64).max(0);
            format!("{:02}:{:02}:{:02}", remaining / 3600, (remaining / 60) % 60, remaining % 60)
        } else {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let local_now = now + (HOUR_OFFSET * 3600);
            format!("{:02}:{:02}:{:02}", (local_now / 3600 % 24).abs(), (local_now / 60 % 60).abs(), (local_now % 60).abs())
        };

        if !paused {
            last_display_time = time_str.clone();
        }

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
        let color_code = if paused { "1;33" } else { "1;31" };

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
                    frame.push_str(&format!("\x1b[{}m", color_code));
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
        thread::sleep(Duration::from_millis(100));
    }
}