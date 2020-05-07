use crate::http::Dataset;
use crate::table::Variable;
use crate::{GraphStore, Resource, Selection};
use rdf::graph::Graph;
use rdf::node::Node;
use rdf::triple::Triple;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use typed_html::dom::DOMTree;
use typed_html::{html, text};

pub async fn export_to_html<'a>(dataset: &'a Dataset<'a>) {
    let concepts_table = dataset
        .select(Selection::of_resources_from_named_graphs())
        .await;
    let concepts_variable = concepts_table.variables.get(0).unwrap();
    let concepts: HashSet<Resource> = concepts_table
        .bindings
        .iter()
        .map(|c| match c.get(&concepts_variable) {
            Some(rdf::node::Node::UriNode { uri: id }) => Resource::from(id.to_string().as_str()),
            n => panic!("Unexpected: {:?}", n),
        })
        .collect();
    let file_names: HashMap<&Resource, String> = concepts
        .iter()
        .map(|r| {
            let mut hasher = Sha256::new();
            hasher.input(&r.0);
            let result = format!("{}.html", hex::encode(hasher.result()));
            (r, result)
        })
        .collect();

    println!("File names: {:?}", file_names);

    println!("Concepts: {:?}", concepts);
    for c in concepts.iter() {
        let links_from = dataset.select(Selection::of_relations_from(c)).await;
        println!("Links from: {:?}", links_from);

        let links_to = dataset.select(Selection::of_relations_to(c)).await;
        println!("Links to: {:?}", links_to);

        fn title(map: &HashMap<Variable, Node>, node: &Variable, label: &Variable) -> String {
            match (map.get(node), map.get(label)) {
                (
                    _,
                    Some(Node::LiteralNode {
                        literal: s,
                        data_type: _,
                        language: _,
                    }),
                ) => s.to_string(),
                (Some(Node::UriNode { uri: u }), _) => u.to_string().clone(),
                (
                    Some(Node::LiteralNode {
                        literal: s,
                        data_type: _,
                        language: _,
                    }),
                    _,
                ) => s.to_string(),
                (a, b) => panic!(
                    "Unexpected labelled node: {:?}, tried {:?} and {:?}, found {:?} and {:?}",
                    map, node, label, a, b
                ),
            }
        }

        let defs = links_from.bindings.iter().map(|b| {
            html!(
                <li>
                    <b>{ text!("{}", title(b, &Variable::from("predicate"), &Variable::from("predicate_label"))) }</b>
                    { text!(" {}", title(b, &Variable::from("object"), &Variable::from("object_label"))) }
                </li>
            )
        });

        let title = &c.0;
        let mut doc: DOMTree<String> = html!(
            <html>
                <head>
                    <title>{ text!("{}", title) }</title>
                </head>
                <body>
                    <h1>{ text!("{}",title) }</h1>
                    <ul>
                        { defs }
                    </ul>
                </body>
            </html>
        );

        fs::write(file_names.get(&c).unwrap(), doc.to_string());
    }
}
