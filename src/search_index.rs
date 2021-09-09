use anyhow::bail;
use reqwest::{Client, Url};
use scraper::{Html, Selector};

pub struct IndexInitializer {
    client: Client,
    crate_name: String,
    crate_version: String,
    base_url: String,
}

impl IndexInitializer {
    pub fn new(crate_name: &str, crate_version: &str) -> Self {
        let client = Client::new();
        let base_url = format!(
            "https://docs.rs/{name}/{version}/{name}/",
            name = crate_name,
            version = crate_version
        );
        Self {
            client,
            crate_name: crate_name.to_string(),
            crate_version: crate_version.to_string(),
            base_url,
        }
    }

    pub async fn fetch_search_index_url(&self) -> anyhow::Result<String> {
        let html = self.fetch_top_page().await?;
        let document = Html::parse_document(&html);
        let filename = self.extract_search_index_filename(&document)?;
        let base_url = Url::parse(&self.base_url)?;
        let url = base_url.join(&filename)?;
        Ok(url.to_string())
    }

    async fn fetch_top_page(&self) -> anyhow::Result<String> {
        Ok(self
            .client
            .get(&self.base_url)
            .send()
            .await?
            .text_with_charset("utf-8")
            .await?)
    }

    fn extract_search_index_filename(&self, html: &Html) -> anyhow::Result<String> {
        // Newer and more information
        let div_tags = Selector::parse("div").unwrap();
        for element in html.select(&div_tags) {
            if let Some(index_filename) = element.value().attr("data-search-index-js") {
                return Ok(index_filename.to_string());
            }
        }

        // Older and less information
        let script_tags = Selector::parse("script").unwrap();
        for element in html.select(&script_tags) {
            if let Some(maybe_index_filename) = element.value().attr("src") {
                if maybe_index_filename.contains("search-index") {
                    return Ok(maybe_index_filename.to_string());
                }
            }
        }

        bail!(
            "Cannot find search index file for {}@{}",
            self.crate_name,
            self.crate_version
        );
    }
}
