use axum::Router;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(app::App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || app::shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(app::shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
