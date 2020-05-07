use graph_store::http::Dataset;
use graph_store::{DataFile, Graph, GraphStore, Selection};
use uuid::Uuid;

#[tokio::test]
async fn create_select_and_delete() {
    let client = reqwest::Client::new();
    let base = url::Url::parse("http://localhost:3030").unwrap();
    let name = format!("test-{}", Uuid::new_v4());
    let file = DataFile::unsafe_from_turtle(
        "@base <urn:uuid:bc36c84d-30bf-4014-9940-255150891034>

<#a> <#b> \"c\" .
<#d> <#e> <#f>",
    );

    let dataset = Dataset::get_or_create(&client, base, &name).await;

    dataset.import(Graph::Default, file).await;

    let result = dataset.select(Selection::of_triples()).await;
    println!("selection: {:?}", result);

    dataset.delete().await;
}
