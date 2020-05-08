use crate::http::Dataset;
use crate::table::Variable;
use crate::{GraphStore, Resource, Selection};
use rdf::node::Node;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use typed_html::dom::DOMTree;
use typed_html::{html, text};

pub async fn export_to_html<'a>(dataset: &'a Dataset<'a>) {
    #[derive(Debug)]
    struct ResourceProperties {
        label: String,
        file_name: String,
    }

    let map: HashMap<Resource, ResourceProperties> = dataset
        .select(Selection::of_resources_with_labels())
        .await
        .bindings
        .into_iter()
        .map(|r| {
            let resource = match r.get(&Variable::from("resource")) {
                Some(rdf::node::Node::UriNode { uri: id }) => id.to_string().clone(),
                _ => panic!("Unexpected resource"),
            };
            (
                Resource::from(resource.as_ref()),
                ResourceProperties {
                    label: match r.get(&Variable::from("label")) {
                        Some(rdf::node::Node::LiteralNode {
                            literal: s,
                            data_type: _,
                            language: _,
                        }) => s.to_string(),
                        _ => resource.to_string(),
                    },
                    file_name: {
                        let mut hasher = Sha256::new();
                        hasher.input(&resource.to_string());
                        format!("{}.html", hex::encode(hasher.result()))
                    },
                },
            )
        })
        .collect();

    fn resource_component(
        map: &HashMap<Resource, ResourceProperties>,
        hash_map: &HashMap<Variable, Node>,
        selector: &Variable,
    ) -> Box<typed_html::elements::a<String>> {
        match hash_map.get(selector) {
            Some(rdf::node::Node::UriNode { uri: u }) => {
                let resource = Resource::from(u.to_string().as_str());
                match map.get(&resource) {
                    Some(props) => {
                        let href = &props.file_name;
                        html!(<a href=href>{ text!("{}", props.label) }</a>)
                    }
                    None => html!(<a>"noprops"</a>),
                }
            }
            Some(rdf::node::Node::LiteralNode {
                literal: s,
                language: _,
                data_type: _,
            }) => html!(<a>{ text!("{}", s) }</a>),
            r => panic!("Unexpected resource {:?}", r),
        }
    }

    for (r, props) in map.iter() {
        let links_from = dataset.select(Selection::of_relations_from(r)).await;
        let links_to = dataset.select(Selection::of_relations_to(r)).await;

        let links_from_html = links_from.bindings.iter().map(|link| {
            html!(
                <li>
                    { resource_component(&map, link, &Variable::from("predicate")) }
                    { text!(" → ") }
                    { resource_component(&map, link, &Variable::from("object")) }
                </li>
            )
        });

        let links_to_html = links_to.bindings.iter().map(|link| {
            html!(
                <li>
                    { resource_component(&map, link, &Variable::from("subject")) }
                    { text!(" ← ") }
                    { resource_component(&map, link, &Variable::from("predicate")) }
                </li>
            )
        });

        let doc: DOMTree<String> = html!(
            <html>
                <head>
                    <title>{ text!("{}", props.label) }</title>
                    <link rel="stylesheet" href="main.css"/>
                </head>
                <body>
                    <h1>{ text!("{}", props.label) }</h1>
                    <ul>
                        { links_from_html }
                        { links_to_html }
                    </ul>
                    <a href="index.html">"Index"</a>
                </body>
            </html>
        );

        fs::write(
            props.file_name.to_string(),
            format!("<!doctype html>{}", doc.to_string()),
        )
        .expect("Could not write file");
    }

    let links = map.iter().map(|(_, props)| {
        let href = &props.file_name;
        let label = &props.label;
        html!(
            <li>
                <a href=href>{ text!("{}", label) }</a>
            </li>
        )
    });

    let doc: DOMTree<String> = html!(
        <html>
            <head>
                <title>"Index"</title>
                <link rel="stylesheet" href="main.css"/>
            </head>
            <body>
                <h1>"Index"</h1>
                <ul>
                    { links }
                </ul>
            </body>
        </html>
    );

    fs::write("index.html", format!("<!doctype html>{}", doc.to_string()))
        .expect("Could not write index");
}
