pub mod http;

use async_trait::async_trait;

/// Any resource, identified by an IRI string.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Resource(String);

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

    pub fn unsafe_from(value: &str) -> Selection {
        Selection {
            sparql_value: value.to_string(),
        }
    }
}

/// A variable in a query result.
#[derive(Debug)]
pub struct Variable(String);

/// Results from a selection query.
#[derive(Debug)]
pub struct SelectionResult {
    pub vars: Vec<Variable>,
    pub bindings: Vec<Vec<rdf::node::Node>>,
}

/// A collection of RDF graphs.
#[async_trait]
pub trait GraphStore {
    /// Imports a file into a dataset.
    async fn import(&self, graph: Graph, file: DataFile);

    /// Performs a SPARQL query.
    async fn select(&self, query: &Selection) -> SelectionResult;
}
