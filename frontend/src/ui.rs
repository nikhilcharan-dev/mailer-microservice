use leptos::prelude::*;

/// Render a Leptos view to a full HTML document string (no hydration / WASM).
pub fn render_page<F, V>(view_fn: F) -> String
where
    F: FnOnce() -> V,
    V: RenderHtml + Send + 'static,
{
    // A reactive Owner gives signals & effects a place to live during render,
    // even when our views are mostly static.
    let owner = Owner::new();
    let html_body = owner.with(|| view_fn().to_html());
    format!("<!DOCTYPE html>{html_body}")
}

#[component]
pub fn Layout(
    #[prop(into)] title: String,
    #[prop(into)] active: String,
    children: Children,
) -> impl IntoView {
    let nav_item = |path: &'static str, label: &'static str| {
        let cls = if active == path { "active" } else { "" };
        view! { <a href={path} class={cls}>{label}</a> }
    };
    view! {
        <html lang="en">
            <head>
                <meta charset="UTF-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <title>{format!("{title} · Central Mailer")}</title>
                <link rel="stylesheet" href="/static/style.css"/>
            </head>
            <body>
                <div class="app">
                    <aside class="sidebar">
                        <div class="brand">"📬 Central Mailer"</div>
                        <nav>
                            {nav_item("/", "Overview")}
                            {nav_item("/transports", "Transports")}
                            {nav_item("/templates", "Templates")}
                            {nav_item("/logs", "Delivery Logs")}
                            {nav_item("/settings", "Settings")}
                        </nav>
                        <div class="logout">
                            <form method="POST" action="/logout">
                                <button type="submit">"Sign out"</button>
                            </form>
                        </div>
                    </aside>
                    <main class="main">
                        {children()}
                    </main>
                </div>
            </body>
        </html>
    }
}

#[component]
pub fn AuthLayout(
    #[prop(into)] title: String,
    children: Children,
) -> impl IntoView {
    view! {
        <html lang="en">
            <head>
                <meta charset="UTF-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <title>{format!("{title} · Central Mailer")}</title>
                <link rel="stylesheet" href="/static/style.css"/>
            </head>
            <body>
                <div class="auth">
                    {children()}
                </div>
            </body>
        </html>
    }
}

#[component]
pub fn Flash(#[prop(into)] kind: String, #[prop(into)] msg: String) -> impl IntoView {
    let cls = format!("flash {kind}");
    view! { <div class={cls}>{msg}</div> }
}
