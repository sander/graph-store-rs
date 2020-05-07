use graph_store::http::Dataset;
use graph_store::{doc, Resource};
use graph_store::{DataFile, DescribeQuery, Graph, GraphStore, Selection};
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::test]
async fn create_select_describe_and_delete() {
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

    let result = dataset
        .describe(DescribeQuery::unsafe_from(
            "DESCRIBE ?x
WHERE {
  ?x ?y ?z
}",
        ))
        .await;
    println!("description: {:?}", result);

    let result = dataset.describe_everything().await;
    println!("description of everything: {:?}", result);

    dataset.delete().await;
}

#[tokio::test]
async fn html_files() {
    let client = reqwest::Client::new();
    let base = url::Url::parse("http://localhost:3030").unwrap();
    let name = format!("test-{}", Uuid::new_v4());
    let file = DataFile::unsafe_from_turtle(
        "@base <urn:uuid:bc36c84d-30bf-4014-9940-255150891034>

<#a> <#b> \"c\" .
<#d> <#e> <#f> .
<#d> <#g> <#a> .",
    );

    let dataset = Dataset::get_or_create(&client, base, &name).await;

    dataset
        .import(Graph::Named(Resource::from("g1")), file)
        .await;

    doc::export_to_html(&dataset).await;

    dataset.delete().await;
}
