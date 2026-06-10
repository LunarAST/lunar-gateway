use worker::*;

pub mod render;
pub mod auth;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/healthz", |_, _| async move { Response::ok("OK") })
        .get_async("/lunar-map.json", |_, ctx| async move {
            let bucket = ctx.bucket("LUNAR_DATA")?;
            let object = bucket.get("lunar-map.json").execute().await?
                .ok_or_else(|| Error::RustError("lunar-map.json not found".into()))?;
            let body = object.body().ok_or_else(|| Error::RustError("Empty object".into()))?;
            let text = body.text().await?;
            Response::ok(text)
        })
        .get_async("/lunar-map.md", |req, ctx| async move {
            let bucket = ctx.bucket("LUNAR_DATA")?;
            let object = bucket.get("lunar-map.json").execute().await?
                .ok_or_else(|| Error::RustError("lunar-map.json not found".into()))?;
            let body = object.body().ok_or_else(|| Error::RustError("Empty object".into()))?;
            let text = body.text().await?;

            // Parse query parameters manually from worker::Url
            let url = req.url()?;
            let query_str = url.query().unwrap_or("");
            let params: std::collections::HashMap<String, String> = parse_query(query_str);

            let options = render::MdOptions {
                summary: params.get("summary").map_or(false, |v| v == "true"),
                scope: params.get("scope").cloned(),
                status: params.get("status").cloned(),
                path: params.get("path").cloned(),
                style: params.get("style").cloned(),
            };

            let md = render::to_markdown(&text, &options)
                .map_err(|e| Error::RustError(e.to_string()))?;
            Response::ok(md)
        })
        .get_async("/private/lunar-map.md", |req, ctx| async move {
            let auth_header = req.headers().get("Authorization")?.unwrap_or_default();
            if !auth_header.starts_with("Bearer ") {
                return Response::error("Unauthorized", 401);
            }
            let token = auth_header.trim_start_matches("Bearer ").trim();
            let public_key_hex = ctx.var("PUBLIC_KEY")?.to_string();
            let valid = auth::verify_token(token, &public_key_hex)?;
            if !valid {
                return Response::error("Forbidden", 403);
            }

            let bucket = ctx.bucket("LUNAR_DATA")?;
            let object = bucket.get("lunar-map.json").execute().await?
                .ok_or_else(|| Error::RustError("lunar-map.json not found".into()))?;
            let body = object.body().ok_or_else(|| Error::RustError("Empty object".into()))?;
            let text = body.text().await?;
            let options = render::MdOptions::default();
            let md = render::to_markdown(&text, &options)
                .map_err(|e| Error::RustError(e.to_string()))?;
            Response::ok(md)
        })
        .run(req, env).await
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}
