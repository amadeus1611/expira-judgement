//! Every research note under docs/research must carry the sections the iterate skill requires,
//! so a fix can always be traced to its sources and their licences.

#[test]
fn research_notes_have_required_sections() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/research");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).expect("docs/research exists") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("md")
            || path.file_name().unwrap() == "README.md"
        {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for h in [
            "## Problem",
            "## Sources",
            "## Licence",
            "## Candidates",
            "## Decision",
        ] {
            assert!(
                text.lines().any(|l| l.trim() == h),
                "{} is missing `{h}`",
                path.display()
            );
        }
        let sources = text
            .split("## Sources")
            .nth(1)
            .unwrap()
            .split("\n## ")
            .next()
            .unwrap();
        assert!(
            sources.contains("http"),
            "{}: Sources has no links",
            path.display()
        );
        checked += 1;
    }
    assert!(checked > 0, "no research notes found");
}
