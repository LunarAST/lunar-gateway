use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

pub struct MdOptions {
    pub summary: bool,
    pub scope: Option<String>,
    pub status: Option<String>,
    pub path: Option<String>,
    pub style: Option<String>,
}

impl Default for MdOptions {
    fn default() -> Self {
        Self {
            summary: false,
            scope: None,
            status: None,
            path: None,
            style: None,
        }
    }
}

pub fn to_markdown(json_str: &str, options: &MdOptions) -> Result<String> {
    let map: Value = serde_json::from_str(json_str)?;

    if options.summary {
        return summary_markdown(&map);
    }

    if options.style.as_deref() == Some("mermaid") {
        return mermaid_markdown(&map);
    }

    let mut md = String::from("# Ecosystem Topology\n\n");

    if let Some(projects) = map["projects"].as_array() {
        let filtered: Vec<&Value> = if let Some(ref scope) = options.scope {
            projects.iter().filter(|p| p["name"].as_str().map_or(false, |n| n.eq_ignore_ascii_case(scope))).collect()
        } else {
            projects.iter().collect()
        };

        for p in filtered {
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
        let filtered: Vec<&Value> = if let Some(ref status) = options.status {
            alignments.iter().filter(|a| a["status"].as_str().map_or(false, |s| s.eq_ignore_ascii_case(status))).collect()
        } else if let Some(ref path) = options.path {
            alignments.iter().filter(|a| a["path"].as_str().map_or(false, |p| p.contains(path))).collect()
        } else {
            alignments.iter().collect()
        };

        if !filtered.is_empty() {
            md.push_str("\n## Alignments\n");
            for a in filtered {
                md.push_str(&format!("- {} → {}: {} {} ({})\n", a["clientProject"].as_str().unwrap_or(""), a["serverProject"].as_str().unwrap_or(""), a["method"].as_str().unwrap_or(""), a["path"].as_str().unwrap_or(""), a["status"].as_str().unwrap_or("")));
            }
        }
    }

    Ok(md)
}

fn summary_markdown(map: &Value) -> Result<String> {
    let mut md = String::from("# Ecosystem Summary\n\n");
    let projects = map["projects"].as_array().map(|a| a.as_slice()).unwrap_or(&[]);

    let total_exposed: usize = projects.iter()
        .map(|p| p["interfaces"]["exposed"].as_array().map_or(0, |a| a.len())).sum();
    let total_consumed: usize = projects.iter()
        .map(|p| p["interfaces"]["consumed"].as_array().map_or(0, |a| a.len())).sum();

    let orphaned = map["anomalies"]["orphanedConsumers"].as_array().map_or(0, |a| a.len());
    let unused = map["anomalies"]["unusedEndpoints"].as_array().map_or(0, |a| a.len());

    md.push_str(&format!("- Projects: {}\n- Total Exposed Endpoints: {}\n- Total Consumed Dependencies: {}\n- Anomalies: {} orphaned, {} unused\n\n", projects.len(), total_exposed, total_consumed, orphaned, unused));
    md.push_str("## Projects\n\n| Name | Type | Exposed | Consumed | Status |\n|:---|:---|:---|:---|:---|\n");

    for p in projects {
        let exp = p["interfaces"]["exposed"].as_array().map_or(0, |a| a.len());
        let con = p["interfaces"]["consumed"].as_array().map_or(0, |a| a.len());
        md.push_str(&format!("| {} | {} | {} | {} | {} |\n", p["name"].as_str().unwrap_or(""), p["type"].as_str().unwrap_or(""), exp, con, p["scanStatus"].as_str().unwrap_or("")));
    }

    if orphaned + unused > 0 {
        md.push_str("\n## Top Risks\n");
        if let Some(orphaned_list) = map["anomalies"]["orphanedConsumers"].as_array() {
            for ep in orphaned_list {
                md.push_str(&format!("1. **Orphaned**: {} calls `{} {}` but target not found.\n", ep["project"].as_str().unwrap_or(""), ep["method"].as_str().unwrap_or(""), ep["path"].as_str().unwrap_or("")));
            }
        }
        if let Some(unused_list) = map["anomalies"]["unusedEndpoints"].as_array() {
            for ep in unused_list {
                md.push_str(&format!("1. **Unused**: {} exposes `{} {}` but no consumer.\n", ep["project"].as_str().unwrap_or(""), ep["method"].as_str().unwrap_or(""), ep["path"].as_str().unwrap_or("")));
            }
        }
    }

    Ok(md)
}

fn mermaid_markdown(map: &Value) -> Result<String> {
    let mut md = String::from("```mermaid\ngraph LR\n");
    let projects = map["projects"].as_array().map(|a| a.as_slice()).unwrap_or(&[]);

    let mut node_ids = HashMap::new();
    for (i, p) in projects.iter().enumerate() {
        if let Some(name) = p["name"].as_str() {
            node_ids.insert(name, i);
            md.push_str(&format!("  n{}[{}]\n", i, name));
        }
    }

    if let Some(edges) = map["aggregatedEdges"].as_array() {
        for edge in edges {
            let from = edge["clientProject"].as_str().unwrap_or("");
            let to = edge["serverProject"].as_str().unwrap_or("");
            if let (Some(&f), Some(&t)) = (node_ids.get(from), node_ids.get(to)) {
                let style = if edge["status"].as_str() == Some("Orphaned") { " -- " } else { " --> " };
                md.push_str(&format!("  n{}{}|{} {}|n{}\n", f, style, edge["callCount"].as_u64().unwrap_or(1), edge["status"].as_str().unwrap_or(""), t));
            }
        }
    }

    md.push_str("```\n");
    Ok(md)
}
