use std::fs;
use std::path::Path;

fn main() {
    const S: &str = "_S_";
    const E: &str = "_E_";

    let src = Path::new("src/main.rs");
    let dest = Path::new("src/output.rs");

    let content = fs::read_to_string(src).expect("Read error");

    let start_k = "fn logic() {";
    let start_i = content.find(start_k).expect("Keyword not found") + start_k.len();
    let end_i = content.rfind("}").expect("Closing brace not found");
    let body = content[start_i..end_i].trim();

    let flattened = body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let protected = flattened
        .replace("r###\"", S)
        .replace("\"###", E);

    let surrogate_body = protected
        .replace("###", "#");

    let body_with_backticks = surrogate_body.replace("#[", "`");

    let injected = surrogate_body
        .replace(
            &format!("{}?{}", S, E),
            &format!("r###\"/* rustc qclock.rs -o qclock && ./qclock [-b] [hh:mm:ss]; Iskaa303 (2026) GNU GPLv3 */ pub fn main() {{ {} }}\"###", body_with_backticks))
        .replace(S, "r#\"")
        .replace(E, "\"#");

    let output_content = format!(
        "/* rustc qclock.rs -o qclock && ./qclock [-b] [hh:mm:ss]; Iskaa303 (2026) GNU GPLv3 */ pub fn main() {{ {} }}",
        injected
    );

    fs::write(dest, output_content).expect("Write error");
    println!("cargo:rerun-if-changed=src/main.rs");
}