pub mod doc;
pub mod http;
pub mod table;

use crate::table::{Table, Variable};
use async_trait::async_trait;
use rdf::node::Node;

/// Any resource, identified by an IRI string.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Resource(String);

impl From<&str> for Resource {
    fn from(s: &str) -> Self {
        Resource(s.to_string())
    }
}

/// The default graph or a named graph of a graph store.
pub enum Graph {
    Default,
    Named(Resource),
}

/// The content of a data file, used for importing.
#[derive(Debug)]
pub enum DataFile {
    Turtle { content: Vec<u8> },
    RdfXml { content: Vec<u8> },
}

impl DataFile {
    pub fn unsafe_from_turtle(s: &str) -> Self {
        DataFile::Turtle {
            content: Vec::from(s.as_bytes()),
        }
    }
}

/// A query to select data.
pub struct Selection {
    sparql_value: String,
}

impl Selection {
    /// Selecting a set of triples.
    pub fn of_triples() -> Selection {
        Selection::unsafe_from("SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 25")
    }

    pub fn of_graphs() -> Selection {
        Selection::unsafe_from("SELECT DISTINCT ?graph WHERE { GRAPH ?graph { ?s ?p ?o } }")
    }

    pub fn of_resources_from_named_graphs() -> Selection {
        Selection::unsafe_from("SELECT ?s WHERE { GRAPH ?g { ?s ?p ?o } }")
    }

    pub fn of_relations_from(resource: &Resource) -> Selection {
        Selection::unsafe_from(&format!(
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?predicate ?predicate_label ?object ?object_label
WHERE {{
  GRAPH ?g1 {{ <{}> ?predicate ?object }}
  OPTIONAL {{ GRAPH ?g2 {{ ?predicate rdfs:label ?predicate_label }} }}
  OPTIONAL {{ GRAPH ?g3 {{ ?object rdfs:label ?object_label }} }}
}}",
            resource.0
        ))
    }

    pub fn of_relations_to(resource: &Resource) -> Selection {
        Selection::unsafe_from(&format!(
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?subject ?subject_label ?predicate ?predicate_label
WHERE {{
  GRAPH ?g1 {{ ?subject ?predicate <{}> }}
  OPTIONAL {{ GRAPH ?g2 {{ ?predicate rdfs:label ?predicate_label }} }}
  OPTIONAL {{ GRAPH ?g3 {{ ?subject rdfs:label ?subject_label }} }}
}}",
            resource.0
        ))
    }

    pub fn of_resources_with_labels() -> Selection {
        Selection::unsafe_from(
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT DISTINCT ?resource ?label
WHERE {
  GRAPH ?graph1 {
    { ?resource ?predicate ?object }
    UNION
    { ?subject ?resource ?object }
    UNION
    { ?subject ?predicate ?resource }
  } .
  OPTIONAL {
    GRAPH ?graph3 { ?resource rdfs:label ?label }
  } .
  FILTER ( isURI(?resource) )
}",
        )
    }

    pub fn unsafe_from(value: &str) -> Selection {
        Selection {
            sparql_value: value.to_string(),
        }
    }
}

pub struct DescribeQuery {
    sparql_value: String,
}

impl DescribeQuery {
    pub fn unsafe_from(value: &str) -> DescribeQuery {
        DescribeQuery {
            sparql_value: value.to_string(),
        }
    }
}

/// A collection of RDF graphs.
#[async_trait]
pub trait GraphStore {
    /// Imports a file into a dataset.
    async fn import(&self, graph: Graph, file: DataFile);

    /// Performs a SPARQL query.
    async fn select(&self, query: Selection) -> Table<Node>;

    async fn describe(&self, query: DescribeQuery) -> rdf::graph::Graph;

    async fn describe_everything(&self) -> rdf::graph::Graph {
        let graphs = self.select(Selection::of_graphs()).await;
        let from: String = graphs
            .bindings
            .iter()
            .map(|b| {
                format!(
                    "FROM <{}>",
                    match b.get(&Variable::from("graph")) {
                        Some(rdf::node::Node::UriNode { uri: id }) => id.to_string(),
                        _ => panic!("Unexpected node"),
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let query =
            DescribeQuery::unsafe_from(&format!("DESCRIBE ?x {} WHERE {{ ?x ?y ?z }}", from));
        self.describe(query).await
    }
}
