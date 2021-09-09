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

    pub async fn fetch_search_index(&self) -> anyhow::Result<String> {
        let url = self.fetch_search_index_url().await?;
        let search_index_js = self
            .client
            .get(url)
            .send()
            .await?
            .text_with_charset("utf-8")
            .await?;
        Self::parse_search_index_js(&search_index_js)
    }

    fn parse_search_index_js(index: &str) -> anyhow::Result<String> {
        // Targetted JSON is surrounded by outermost `'` pair.
        let json_start = index
            .find('\'')
            .ok_or_else(|| anyhow::anyhow!("Invalid format of JSON search index"))?
            + 1;
        let json_end = index
            .rfind('\'')
            .ok_or_else(|| anyhow::anyhow!("Invalid format of JSON search index"))?;
        let json = &index[json_start..json_end];
        Ok(IndexInitializer::format_json(json))
    }

    fn format_json(index: &str) -> String {
        // JSON parsed by `parse_search_index_js` includes unnecessary `\` because the JSON is
        // originally embedded in JavaScript code as a string literal.
        // So remove `\`s along following constraints:
        // * `\"` -> `\"`
        // * `\{ANY CHARACTER}` -> `{ANY CHARACTER}`
        let mut chars = index.chars().peekable();
        let mut formatted = String::new();
        while let Some(c) = chars.next() {
            if c == '\\' && chars.peek() != Some(&'\"') {
                continue;
            }
            formatted.push(c);
        }
        formatted
    }

    async fn fetch_search_index_url(&self) -> anyhow::Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_search_index_json() {
        let json = r#"{ "desc": "I have a \\\"dream\\\".", "hoge": "John\'s"}"#;
        assert_eq!(
            r#"{ "desc": "I have a \"dream\".", "hoge": "John's"}"#,
            IndexInitializer::format_json(json)
        )
    }
}
