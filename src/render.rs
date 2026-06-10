use anyhow::Result;
use serde_json::Value;

pub fn to_markdown(json_str: &str) -> Result<String> {
    let map: Value = serde_json::from_str(json_str)?;
    let mut md = String::from("# Ecosystem Topology\n\n");

    if let Some(projects) = map["projects"].as_array() {
        for p in projects {
            md.push_str(&format!("## {}\n", p["name"].as_str().unwrap_or("")));
            if let Some(interfaces) = p["interfaces"].as_object() {
                if let Some(exposed) = interfaces["exposed"].as_array() {
                    md.push_str("### Exposed\n");
                    for e in exposed {
                        md.push_str(&format!("- {} {}\n", e["method"].as_str().unwrap_or(""), e["path"].as_str().unwrap_or("")));
                    }
                }
                if let Some(consumed) = interfaces["consumed"].as_array() {
                    md.push_str("### Consumed\n");
                    for c in consumed {
                        md.push_str(&format!("- {} {} -> {}\n", c["method"].as_str().unwrap_or(""), c["path"].as_str().unwrap_or(""), c["targetProject"].as_str().unwrap_or("?")));
                    }
                }
            }
        }
    }

    if let Some(alignments) = map["alignments"].as_array() {
        md.push_str("\n## Alignments\n");
        for a in alignments {
            md.push_str(&format!("- {} → {}: {} {} ({})\n", a["clientProject"].as_str().unwrap_or(""), a["serverProject"].as_str().unwrap_or(""), a["method"].as_str().unwrap_or(""), a["path"].as_str().unwrap_or(""), a["status"].as_str().unwrap_or("")));
        }
    }

    Ok(md)
}
