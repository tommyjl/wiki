#![allow(dead_code)]

use crate::article_provider::{ArticleProvider, LocalArticleProvider};
use axum::{extract::Path, handler::get, http::HeaderMap, Router};
use http::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use pulldown_cmark::{html, Parser};
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct PalaverConfig {
    article_dir: PathBuf,
}

mod article_provider;

#[tokio::main]
async fn main() {
    // TODO: Use config
    // TODO: Render html title and header from config
    let _config = PalaverConfig {
        article_dir: "../articles".into(),
    };

    // TODO: 404 page
    // TODO: Request logging middleware
    let app = Router::new()
        .route("/", get(list_articles))
        .route("/wiki/:id", get(show_article).post(edit_article))
        .route("/wiki/:id/raw", get(show_raw_article))
        .route("/styles.css", get(stylesheet));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn render_html(body: &str) -> String {
    let html = include_str!("www/index.html");
    let html = html.replace("{{main}}", body);
    html
}

fn render_html_from_markdown(md: &str) -> String {
    let mut html_output = String::new();
    let parser = Parser::new(md);
    html::push_html(&mut html_output, parser);

    render_html(&html_output)
}

async fn list_articles() -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::with_capacity(10);
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/html"),
    );

    let provider = LocalArticleProvider::new("articles".into());
    let articles = provider.list_articles().await.unwrap_or_default();

    let mut html_body = String::new();
    for article in articles {
        html_body.push_str(&format!(
            "<li><a href=\"wiki/{}\">{}</a></li>\n",
            article, article
        ));
    }
    if html_body.is_empty() {
        html_body.push_str("No articles\n");
    }
    let html = render_html(&html_body);

    (StatusCode::OK, headers, html)
}

async fn show_article(Path(article_id): Path<String>) -> (HeaderMap, String) {
    let mut headers = HeaderMap::with_capacity(10);
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/html"),
    );

    let provider = LocalArticleProvider::new("articles".into());
    let article = provider.show_article(&article_id).await;

    let html_body = article.unwrap_or_else(|()| "Not found".to_string());
    let html = render_html_from_markdown(&html_body);

    (headers, html)
}

async fn show_raw_article(Path(article_id): Path<String>) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::with_capacity(10);
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/markdown"),
    );

    let provider = LocalArticleProvider::new("articles".into());
    match provider.show_article(&article_id).await {
        Ok(article) => (StatusCode::OK, headers, article),
        Err(_) => (StatusCode::NOT_FOUND, headers, String::new()),
    }
}

async fn edit_article(Path(_article_id): Path<String>) -> &'static str {
    todo!()
}

async fn stylesheet() -> (HeaderMap, &'static str) {
    let css = include_str!("www/styles.css");
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/css"),
    );
    (headers, css)
}
