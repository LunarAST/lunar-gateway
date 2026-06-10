use worker::*;

mod render;

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
        .get_async("/lunar-map.md", |_, ctx| async move {
            let bucket = ctx.bucket("LUNAR_DATA")?;
            let object = bucket.get("lunar-map.json").execute().await?
                .ok_or_else(|| Error::RustError("lunar-map.json not found".into()))?;
            let body = object.body().ok_or_else(|| Error::RustError("Empty object".into()))?;
            let text = body.text().await?;
            let md = render::to_markdown(&text)
                .map_err(|e| Error::RustError(e.to_string()))?;
            Response::ok(md)
        })
        .run(req, env).await
}
