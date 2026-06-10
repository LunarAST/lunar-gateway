use lunar_gateway::render::{to_markdown, MdOptions};

#[test]
fn test_summary_mode() {
    let json = r#"{
        "projects": [
            {
                "name": "test-service",
                "type": "service",
                "scanStatus": "success",
                "interfaces": {
                    "exposed": [{"path": "/api/health", "method": "GET"}],
                    "consumed": [{"path": "/api/auth", "method": "POST", "targetProject": "auth-svc"}]
                }
            }
        ],
        "alignments": [
            {
                "clientProject": "test-service",
                "serverProject": "auth-svc",
                "path": "/api/auth",
                "method": "POST",
                "status": "Aligned"
            }
        ],
        "aggregatedEdges": [],
        "anomalies": {
            "unusedEndpoints": [],
            "orphanedConsumers": [],
            "crossLayerViolations": []
        }
    }"#;
    let options = MdOptions { summary: true, ..Default::default() };
    let result = to_markdown(json, &options).unwrap();
    assert!(result.contains("test-service"));
    assert!(result.contains("1"));
}

#[test]
fn test_scope_filter() {
    let json = r#"{
        "projects": [
            {"name": "svc-a", "type": "service", "scanStatus": "success", "interfaces": {"exposed":[], "consumed":[]}},
            {"name": "svc-b", "type": "service", "scanStatus": "success", "interfaces": {"exposed":[], "consumed":[]}}
        ],
        "alignments": [],
        "aggregatedEdges": [],
        "anomalies": {
            "unusedEndpoints": [],
            "orphanedConsumers": [],
            "crossLayerViolations": []
        }
    }"#;
    let options = MdOptions { scope: Some("svc-a".into()), ..Default::default() };
    let result = to_markdown(json, &options).unwrap();
    assert!(result.contains("svc-a"));
    assert!(!result.contains("svc-b"));
}

#[test]
fn test_status_filter() {
    let json = r#"{
        "projects": [],
        "alignments": [
            {"clientProject":"a","serverProject":"b","path":"/api","method":"GET","status":"Aligned"},
            {"clientProject":"c","serverProject":"d","path":"/api","method":"POST","status":"Orphaned"}
        ],
        "aggregatedEdges": [],
        "anomalies": { "unusedEndpoints":[], "orphanedConsumers":[], "crossLayerViolations":[] }
    }"#;
    let options = lunar_gateway::render::MdOptions {
        status: Some("Orphaned".into()),
        ..Default::default()
    };
    let result = lunar_gateway::render::to_markdown(json, &options).unwrap();
    assert!(result.contains("Orphaned"));
    assert!(!result.contains("Aligned"));
}

#[test]
fn test_mermaid_mode() {
    let json = r#"{
        "projects": [
            {"name":"svc-a","type":"service","scanStatus":"success","interfaces":{"exposed":[],"consumed":[]}},
            {"name":"svc-b","type":"service","scanStatus":"success","interfaces":{"exposed":[],"consumed":[]}}
        ],
        "alignments": [],
        "aggregatedEdges": [
            {"clientProject":"svc-a","serverProject":"svc-b","callCount":1,"status":"Aligned","paths":[]}
        ],
        "anomalies": { "unusedEndpoints":[], "orphanedConsumers":[], "crossLayerViolations":[] }
    }"#;
    let options = lunar_gateway::render::MdOptions {
        style: Some("mermaid".into()),
        ..Default::default()
    };
    let result = lunar_gateway::render::to_markdown(json, &options).unwrap();
    assert!(result.contains("```mermaid"));
    assert!(result.contains("svc-a"));
    assert!(result.contains("svc-b"));
}

#[test]
fn test_empty_data_does_not_panic() {
    let json = r#"{"projects":[],"alignments":[],"aggregatedEdges":[],"anomalies":{"unusedEndpoints":[],"orphanedConsumers":[],"crossLayerViolations":[]}}"#;
    let options = lunar_gateway::render::MdOptions::default();
    let result = lunar_gateway::render::to_markdown(json, &options);
    assert!(result.is_ok());
}
