use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs::read_to_string;

pub struct ArticleInfo {
    title: String,
}

#[async_trait]
pub trait ArticleProvider {
    async fn list_articles(&self) -> Result<Vec<String>, ()>;
    async fn show_article(&self, id: &str) -> Result<String, ()>;
}

/// Provides articles from a directory on the local filesystem
pub struct LocalArticleProvider {
    dir: PathBuf,
}

impl LocalArticleProvider {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

#[async_trait]
impl ArticleProvider for LocalArticleProvider {
    async fn list_articles(&self) -> Result<Vec<String>, ()> {
        let ret = std::fs::read_dir(&self.dir)
            .map_err(|_| ())?
            .map(|inner| {
                String::from(
                    inner
                        .unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        .strip_suffix(".md")
                        .unwrap(),
                )
            })
            .collect();
        Ok(ret)
    }

    async fn show_article(&self, id: &str) -> Result<String, ()> {
        let mut file_path = self.dir.clone();
        file_path.push(format!("{}.md", id));
        read_to_string(file_path).await.or(Err(()))
    }
}
