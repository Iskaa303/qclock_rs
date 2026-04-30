use std::fs;
use std::path::Path;

fn main() {
    let src = Path::new("src/main.rs");
    let dest = Path::new("src/output.rs");

    let content = fs::read_to_string(src).expect("Unable to read file");

    let start_k = "fn logic() {";
    let start_i = content.find(start_k).expect("Keyword not found") + start_k.len();
    let end_i = content.rfind("}").expect("Closing brace not found");
    let body = content[start_i..end_i].trim();

    let body = body
        .replace("    ", " ")
        .replace("  ", " ")
        .replace('\n', "")
        .replace('\r', "")
        .replace('\t', "");

    let escaped = body
        .replace('\\', "\\\\")
        .replace('\"', "\\\"");

    let escaped = format!(
        "/* GENERATED FILE - DO NOT EDIT */ fn main() {{{}}}",
        escaped
    );

    let final_code = body.replace("?", &escaped);

    let output_content = format!(
        "/* GENERATED FILE - DO NOT EDIT */ pub fn main() {{{}}}",
        final_code
    );

    let output_content = output_content.replace("?", &escaped);

    fs::write(dest, output_content).expect("Could not write output.rs");
}