//! docs/HABITS.md must keep its two lists, and every retired habit must say why, show evidence and name its replacement,
//! so reports can be checked against it (owner decision 2026-10-11).

#[test]
fn habits_doc_has_adopted_and_retired_with_evidence() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/HABITS.md");
    let text = std::fs::read_to_string(path).expect("docs/HABITS.md exists");
    let section = |name: &str| {
        text.split(&format!("## {name}\n"))
            .nth(1)
            .map(|s| s.split("\n## ").next().unwrap_or("").to_string())
    };
    let adopted = section("Adopted").expect("an Adopted section");
    let retired = section("Retired").expect("a Retired section");
    assert!(
        adopted.lines().any(|l| l.starts_with("- **")),
        "Adopted lists no habits"
    );
    let entries: Vec<&str> = retired.split("\n- **").skip(1).collect();
    assert!(!entries.is_empty(), "Retired lists no habits");
    for e in entries {
        for key in ["Why retired:", "Evidence:", "Replaced by:"] {
            assert!(
                e.contains(key),
                "retired habit missing `{key}`: {}",
                e.lines().next().unwrap_or("")
            );
        }
    }
}
