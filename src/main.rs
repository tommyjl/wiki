#![allow(dead_code)]

use crate::article_provider::{DynArticleProvider, LocalArticleProvider};
use crate::response::{Css, Markdown};
use axum::{
    extract::{Extension, Path},
    handler::{get, Handler},
    response::{Html, IntoResponse},
    AddExtensionLayer, Router,
};
use handlebars::Handlebars;
use http::StatusCode;
use pulldown_cmark::{html, Parser};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

mod article_provider;
mod response;

const TEMPLATE: &str = "layout";
const BODY: &str = "main";

#[derive(Clone)]
pub struct WikiConfig {
    title: String,
    article_dir: PathBuf,
    not_found_msg: String,
}

#[derive(serde::Serialize)]
pub struct Page<'a> {
    title: &'a str,
    main: Option<&'a str>,
    links: Vec<PageLink>,
}

// TODO: Non-heap strings?
#[derive(Debug, serde::Serialize)]
pub struct PageLink {
    href: String,
    text: String,
}

#[tokio::main]
async fn main() {
    // TODO: Get config from environment
    let config = WikiConfig {
        title: "Tommyjl Wiki".into(),
        article_dir: "articles".into(),
        not_found_msg: "404 NotFound".into(),
    };

    // TODO: 404 page
    // TODO: Request logging middleware
    let app = Router::new()
        .route("/", get(list_articles))
        .route("/wiki/:id", get(show_article).post(edit_article))
        .route("/wiki/:id/raw", get(show_raw_article))
        .route("/styles.css", get(stylesheet))
        .layer(AddExtensionLayer::new(init_handlebars()))
        .layer(AddExtensionLayer::new(init_article_provider(&config)))
        .layer(AddExtensionLayer::new(config));

    let app = app.or(handle_not_found.into_service());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_article_provider(config: &WikiConfig) -> DynArticleProvider {
    Arc::new(LocalArticleProvider::new(config.article_dir.clone()))
}

fn init_handlebars() -> Handlebars<'static> {
    let template = include_str!("www/index.html");
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(TEMPLATE, template)
        .unwrap();
    handlebars
}

// TODO: The example used Arc. Investigate if we should use that here.
fn render_html(
    config: &WikiConfig,
    handlebars: &Handlebars,
    body: Option<&str>,
    links: Option<Vec<PageLink>>,
) -> String {
    let data = Page {
        title: &config.title,
        main: body,
        links: links.unwrap_or_else(Vec::new),
    };
    handlebars.render(TEMPLATE, &data).unwrap()
}

fn render_html_from_markdown(config: &WikiConfig, handlebars: &Handlebars, md: &str) -> String {
    let mut html_output = String::new();
    let parser = Parser::new(md);
    html::push_html(&mut html_output, parser);

    render_html(config, handlebars, Some(&html_output), None)
}

async fn list_articles(
    Extension(config): Extension<WikiConfig>,
    Extension(handlebars): Extension<Handlebars<'_>>,
    Extension(provider): Extension<DynArticleProvider>,
) -> Html<String> {
    let articles = provider.list_articles().await.unwrap_or_default();

    let html_body = articles.is_empty().then(|| "No articles");

    let links = articles
        .iter()
        .map(|a| PageLink {
            href: format!("wiki/{}", a),
            text: a.to_string(),
        })
        .collect();

    let html = render_html(&config, &handlebars, html_body, Some(links));

    Html(html)
}

async fn show_article(
    Path(article_id): Path<String>,
    Extension(config): Extension<WikiConfig>,
    Extension(handlebars): Extension<Handlebars<'_>>,
    Extension(provider): Extension<DynArticleProvider>,
) -> Html<String> {
    let article = provider.show_article(&article_id).await;
    let html_body = article.unwrap_or_else(|()| "Not found".to_string());
    let html = render_html_from_markdown(&config, &handlebars, &html_body);
    Html(html)
}

async fn show_raw_article(
    Path(article_id): Path<String>,
    Extension(provider): Extension<DynArticleProvider>,
) -> (StatusCode, Markdown<String>) {
    match provider.show_article(&article_id).await {
        Ok(article) => (StatusCode::OK, Markdown(article)),
        Err(_) => (StatusCode::NOT_FOUND, Markdown(String::new())),
    }
}

async fn edit_article(
    Path(article_id): Path<String>,
    handlebars: Extension<Handlebars<'_>>,
    Extension(config): Extension<WikiConfig>,
    Extension(provider): Extension<DynArticleProvider>,
) -> Html<String> {
    let article = provider.show_article(&article_id).await;
    let html_body = article.unwrap_or_else(|()| "Not found".to_string());
    let html = render_html_from_markdown(&config, &handlebars, &html_body);
    Html(html)
}

async fn stylesheet() -> Css<&'static str> {
    Css(include_str!("www/styles.css"))
}

async fn handle_not_found(
    Extension(config): Extension<WikiConfig>,
    Extension(handlebars): Extension<Handlebars<'_>>,
) -> impl IntoResponse {
    let html = render_html(&config, &handlebars, Some(&config.not_found_msg), None);
    Html(html)
}
