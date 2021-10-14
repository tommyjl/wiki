#![allow(dead_code)]

use crate::article_provider::{ArticleProvider, LocalArticleProvider};
use crate::response::{Css, Markdown};
use axum::{
    extract::{Extension, Path},
    handler::get,
    response::Html,
    AddExtensionLayer, Router,
};
use handlebars::Handlebars;
use http::StatusCode;
use pulldown_cmark::{html, Parser};
use std::net::SocketAddr;
use std::path::PathBuf;

mod article_provider;
mod response;

const TEMPLATE: &str = "layout";
const BODY: &str = "main";

#[derive(Clone)]
pub struct PalaverConfig {
    article_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    // TODO: Use config
    // TODO: Render html title and header from config
    let config = PalaverConfig {
        article_dir: "../articles".into(),
    };

    // TODO: 404 page
    // TODO: Request logging middleware
    let app = Router::new()
        .route("/", get(list_articles))
        .route("/wiki/:id", get(show_article).post(edit_article))
        .route("/wiki/:id/raw", get(show_raw_article))
        .route("/styles.css", get(stylesheet))
        .layer(AddExtensionLayer::new(init_handlebars()))
        .layer(AddExtensionLayer::new(config));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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
fn render_html(handlebars: &Handlebars, body: &str) -> String {
    let mut data = std::collections::HashMap::new();
    data.insert(BODY, body);
    handlebars.render(TEMPLATE, &data).unwrap()
}

fn render_html_from_markdown(handlebars: &Handlebars, md: &str) -> String {
    let mut html_output = String::new();
    let parser = Parser::new(md);
    html::push_html(&mut html_output, parser);

    render_html(handlebars, &html_output)
}

async fn list_articles(handlebars: Extension<Handlebars<'_>>) -> Html<String> {
    // TODO: Extract to extenison
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
    let html = render_html(&handlebars, &html_body);

    Html(html)
}

async fn show_article(
    Path(article_id): Path<String>,
    handlebars: Extension<Handlebars<'_>>,
) -> Html<String> {
    // TODO: Extract to extenison
    let provider = LocalArticleProvider::new("articles".into());
    let article = provider.show_article(&article_id).await;

    // TODO: Link to /wiki/:id/edit
    let html_body = article.unwrap_or_else(|()| "Not found".to_string());
    let html = render_html_from_markdown(&handlebars, &html_body);

    Html(html)
}

async fn show_raw_article(Path(article_id): Path<String>) -> (StatusCode, Markdown<String>) {
    // TODO: Extract to extenison
    let provider = LocalArticleProvider::new("articles".into());
    match provider.show_article(&article_id).await {
        Ok(article) => (StatusCode::OK, Markdown(article)),
        Err(_) => (StatusCode::NOT_FOUND, Markdown(String::new())),
    }
}

async fn edit_article(
    Path(article_id): Path<String>,
    handlebars: Extension<Handlebars<'_>>,
) -> Html<String> {
    // TODO: Extract to extenison
    let provider = LocalArticleProvider::new("articles".into());
    let article = provider.show_article(&article_id).await;

    let html_body = article.unwrap_or_else(|()| "Not found".to_string());
    let html = render_html_from_markdown(&handlebars, &html_body);

    Html(html)
}

async fn stylesheet() -> Css<&'static str> {
    Css(include_str!("www/styles.css"))
}
