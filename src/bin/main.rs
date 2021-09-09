use drc::search_index::IndexInitializer;

#[tokio::main]
async fn main() {
    let crate_name = "tokio";
    let crate_version = "latest";
    let init = IndexInitializer::new(crate_name, crate_version);
    let index = init.fetch_search_index_url()
        .await
        .unwrap();
    dbg!(index);
}
