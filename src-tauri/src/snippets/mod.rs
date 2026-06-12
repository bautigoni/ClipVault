//! Snippets module. CRUD lives in `db::repo`; this module wires language defaults and
//! any future per-language helpers.

pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "sql",
    "bash",
    "powershell",
    "python",
    "javascript",
    "typescript",
    "rust",
    "go",
    "json",
    "yaml",
    "markdown",
    "plaintext",
];

pub fn default_extension(language: &str) -> &'static str {
    match language {
        "sql" => "sql",
        "bash" => "sh",
        "powershell" => "ps1",
        "python" => "py",
        "javascript" => "js",
        "typescript" => "ts",
        "rust" => "rs",
        "go" => "go",
        "json" => "json",
        "yaml" => "yml",
        "markdown" => "md",
        _ => "txt",
    }
}
