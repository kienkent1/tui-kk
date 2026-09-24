use std::fs;

fn main() {
    let cargo_toml_str = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    let parsed: toml::Value = toml::from_str(&cargo_toml_str).expect("Failed to parse Cargo.toml");

    println!(
        "cargo:warning=\x1b[1;36m==================== [BUILD.RS METADATA] ====================\x1b[0m"
    );

    if let Some(metadata) = parsed
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.as_table())
    {
        for (key, value) in metadata {
            let env_key = key.to_uppercase();

            let env_val = match value {
                toml::Value::String(s) => s.clone(),
                toml::Value::Integer(i) => i.to_string(),
                toml::Value::Boolean(b) => b.to_string(),
                toml::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|item| item.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                _ => continue,
            };

            println!("cargo:rustc-env={}={}", env_key, env_val);

            println!(
                "cargo:warning=  \x1b[1;34m{:<15}\x1b[0m \x1b[36m->\x1b[0m \x1b[1;32m{}\x1b[0m",
                env_key, env_val
            );
        }
    }

    println!(
        "cargo:warning=\x1b[1;36m=============================================================\x1b[0m"
    );
    println!("cargo:rerun-if-changed=Cargo.toml");
}
