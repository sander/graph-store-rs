use crate::{DataFile, DescribeQuery, Graph, GraphStore, Resource, Selection};

use crate::table::Table;
use async_trait::async_trait;
use rdf::node::Node;
use rdf::reader::rdf_parser::RdfParser;
use rdf::reader::turtle_parser::TurtleParser;
use serde::Deserialize;
use std::collections::HashMap;

/// Implementation of https://www.w3.org/TR/sparql11-http-rdf-update/
pub struct Dataset<'a> {
    client: &'a reqwest::Client,
    base: url::Url,
    name: String,
}

impl DataFile {
    fn multipart(self) -> reqwest::multipart::Part {
        match self {
            DataFile::Turtle { content: contents } => reqwest::multipart::Part::bytes(contents)
                .file_name("file.ttl")
                .mime_str("text/turtle")
                .unwrap(),
            DataFile::RdfXml { content: contents } => reqwest::multipart::Part::bytes(contents)
                .file_name("file.xml")
                .mime_str("text/xml")
                .unwrap(),
        }
    }
}

impl Dataset<'_> {
    /// Returns a named dataset in the graph store.
    pub async fn get_or_create<'a>(
        client: &'a reqwest::Client,
        base: url::Url,
        name: &str,
    ) -> Dataset<'a> {
        match client
            .post(base.join("/$/datasets").unwrap())
            .form(&[("dbName", name), ("dbType", &"mem".to_string())])
            .send()
            .await
            .unwrap()
            .status()
        {
            reqwest::StatusCode::CONFLICT | reqwest::StatusCode::OK => Dataset {
                client,
                base,
                name: name.to_string(),
            },
            _ => panic!("Error creating dataset {}.", name),
        }
    }

    /// Deletes a dataset. Moves the variable so that it cannot be used again.
    pub async fn delete(self) {
        let path = self
            .base
            .join("/$/datasets/")
            .unwrap()
            .join(&self.name)
            .unwrap();
        match self.client.delete(path).send().await.unwrap().status() {
            reqwest::StatusCode::OK => (),
            code => panic!("Unexpected status {}.", code),
        }
    }
}

#[derive(Deserialize, Debug)]
struct QueryResponseHead {
    vars: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[allow(non_camel_case_types)]
enum QueryResponseValue {
    uri { value: String },
    literal { value: String },
}

impl QueryResponseValue {
    fn to_node(&self) -> rdf::node::Node {
        match self {
            QueryResponseValue::uri { value } => rdf::node::Node::UriNode {
                uri: rdf::uri::Uri::new(value.to_string()),
            },
            QueryResponseValue::literal { value } => rdf::node::Node::LiteralNode {
                literal: value.to_string(),
                data_type: None,
                language: None,
            },
        }
    }
}

#[derive(Deserialize, Debug)]
struct QueryResponseResults {
    bindings: Vec<HashMap<String, QueryResponseValue>>,
}

#[derive(Deserialize, Debug)]
struct QueryResponse {
    head: QueryResponseHead,
    results: QueryResponseResults,
}

#[async_trait]
impl GraphStore for Dataset<'_> {
    async fn import(&self, graph: Graph, file: DataFile) {
        let form = reqwest::multipart::Form::new().part("files[]", file.multipart());
        let path = self.base.join(&format!("/{}/data", &self.name)).unwrap();
        let query = match graph {
            Graph::Named(Resource(id)) => vec![("graph", id)],
            Graph::Default => Vec::<(&str, String)>::new(),
        };
        let response = self
            .client
            .put(path)
            .query(&query)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        match status {
            reqwest::StatusCode::CREATED | reqwest::StatusCode::OK => (),
            code => panic!("Unexpected status {} with message {}.", code, body),
        };
    }

    async fn select(&self, selection: Selection) -> Table<Node> {
        let form = [("query", selection.sparql_value)];
        let path = self.base.join(&self.name).unwrap();
        let response = self.client.post(path).form(&form).send().await.unwrap();
        let status = response.status();
        let response: QueryResponse = response.json::<QueryResponse>().await.unwrap();
        match status {
            reqwest::StatusCode::OK => {
                Table::from(response.head.vars, response.results.bindings, |b| {
                    b.to_node()
                })
            }
            code => panic!("Unexpected status {}.", code),
        }
    }

    async fn describe(&self, query: DescribeQuery) -> rdf::graph::Graph {
        let form = [("query", query.sparql_value)];
        let path = self.base.join(&self.name).unwrap();
        let response = self.client.post(path).form(&form).send().await.unwrap();
        let status = response.status();
        let response: String = response.text().await.unwrap();
        match status {
            reqwest::StatusCode::OK => {
                let mut reader = TurtleParser::from_string(response);

                match reader.decode() {
                    Ok(graph) => graph,
                    Err(e) => panic!("Could not parse: {:?}", e),
                }
            }
            code => panic!("Unexpected status {}.", code),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::Dataset;
    use mockito::{mock, Mock};

    fn create_endpoint_mock() -> Mock {
        mock("POST", "/$/datasets").with_status(200).create()
    }

    #[tokio::test]
    async fn correct_create_or_get_request() {
        let client = reqwest::Client::new();
        let base = url::Url::parse(&mockito::server_url()).unwrap();
        let create_mock = create_endpoint_mock();
        let name = "test";

        let dataset = Dataset::get_or_create(&client, base, &name).await;

        create_mock.assert();
        assert_eq!(dataset.name, name);
    }

    #[tokio::test]
    async fn correct_delete_request() {
        let client = reqwest::Client::new();
        let base = url::Url::parse(&mockito::server_url()).unwrap();
        let name = "test";
        let _create_mock = create_endpoint_mock();
        let delete_mock = mock("DELETE", format!("/$/datasets/{}", name).as_ref())
            .with_status(200)
            .create();
        let dataset = Dataset::get_or_create(&client, base, &name).await;

        dataset.delete().await;

        delete_mock.assert();
    }
}
