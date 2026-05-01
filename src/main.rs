mod output;

fn main() {
    output::main();
}

#[allow(dead_code)]
fn logic() {
    const W: usize = 150;
    const H: usize = 30;

    const HASH: char = 35 as char;
    const SPACE: char = 32 as char;
    const NEWLINE: char = 10 as char;
    
    let s = r###"?"###;
    
    let mut expanded = String::new();
    for c in s.chars() {
        match c {
            HASH => expanded.push_str("\x23\x23\x23"),
            _ => expanded.push(c),
        }
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = expanded.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == SPACE {
            tokens.push(SPACE.to_string());
            i += 1;
        } else {
            let mut token = String::new();
            while i < chars.len() && chars[i] != SPACE {
                token.push(chars[i]);
                i += 1;
            }
            tokens.push(token);
        }
    }

    let mut final_output = String::new();
    let mut current_line = Vec::new();
    let mut current_width = 0;
    let mut line_count = 0;

    let mut push_line = |tokens_in_line: Vec<String>, justify: bool| {
        let mut line_str = String::new();
        if justify {
            let content_len: usize = tokens_in_line.iter().map(|t| t.chars().count()).sum();
            let space_tokens = tokens_in_line.iter().filter(|t| t == &&SPACE.to_string()).count();
            
            if space_tokens > 0 {
                let total_extra = W - content_len;
                let mut added = 0;
                let mut space_idx = 0;
                for t in tokens_in_line {
                    if t == SPACE.to_string() {
                        space_idx += 1;
                        let target_extra = (total_extra * space_idx) / space_tokens;
                        let to_add = target_extra - added;
                        line_str.push(SPACE);
                        for _ in 0..to_add { line_str.push(SPACE); }
                        added += to_add;
                    } else {
                        line_str.push_str(&t);
                    }
                }
            } else {
                for t in tokens_in_line { line_str.push_str(&t); }
            }
        } else {
            for t in tokens_in_line { line_str.push_str(&t); }
        }

        while line_str.chars().count() < W { line_str.push(SPACE); }
        let truncated: String = line_str.chars().take(W).collect();
        final_output.push_str(&truncated);
        final_output.push(NEWLINE);
    };

    for t in tokens {
        let t_len = t.chars().count();
        if current_width + t_len > W && !current_line.is_empty() {
            push_line(current_line.clone(), true);
            line_count += 1;
            current_line.clear();
            current_width = 0;
        }
        if !(current_line.is_empty() && t == SPACE.to_string()) {
            current_width += t_len;
            current_line.push(t);
        }
    }

    if !current_line.is_empty() && line_count < H {
        push_line(current_line, false);
        line_count += 1;
    }

    while line_count < H {
        final_output.push_str(&SPACE.to_string().repeat(W));
        final_output.push(NEWLINE);
        line_count += 1;
    }

    print!("{}", final_output);
}