mod output;

fn main() {
    output::main();
}

#[allow(dead_code)]
fn logic() {
    let s = "?";
    for c in s.chars() {
        match c {
            '\\' => print!("\\\\"),
            '"' => print!("\\\""),
            _ => print!("{}", c),
        }
    }
}
