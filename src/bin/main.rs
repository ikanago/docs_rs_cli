use drc::search_index;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let crate_name = "tokio";
    let crate_version = "latest";
    let html = search_index::fetch_search_index(&client, crate_name, crate_version)
        .await
        .unwrap();
}
