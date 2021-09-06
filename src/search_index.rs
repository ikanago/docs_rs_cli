use reqwest::Client;
use scraper::{Html, Selector};

pub async fn fetch_search_index(client: &Client, crate_name: &str, crate_version: &str) -> anyhow::Result<String> {
    let html = fetch_top_page(client, crate_name, crate_version).await?;
    dbg!("Fetched index.html");
    let document = Html::parse_document(&html);
    let script_tag = Selector::parse("div").unwrap();

    for element in document.select(&script_tag) {
        if let Some(attr) = element.value().attr("data-search-index-js") {
            println!("{}", attr);
        }
    }
    Ok(html)
}

async fn fetch_top_page(client: &Client, crate_name: &str, crate_version: &str) -> anyhow::Result<String> {
    let url = format!("https://docs.rs/{name}/{version}/{name}/index.html", name = crate_name, version = crate_version);
    Ok(client.get(&url).send().await?.text_with_charset("utf-8").await?)
}
