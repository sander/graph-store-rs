use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

#[derive(Debug, Eq, PartialEq)]
pub struct Variable {
    name: String,
}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&str> for Variable {
    fn from(s: &str) -> Self {
        Variable {
            name: s.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Table<A> {
    pub variables: Vec<Variable>,
    pub bindings: Vec<HashMap<Variable, A>>,
}

impl<A> Table<A> {
    pub fn from<B>(
        variables: Vec<String>,
        bindings: Vec<HashMap<String, B>>,
        transform: fn(B) -> A,
    ) -> Table<A> {
        Table {
            bindings: bindings
                .into_iter()
                .map(|mut binding| {
                    binding
                        .into_iter()
                        .map(|(k, v)| (Variable::from(k.as_str()), transform(v)))
                        .collect()
                })
                .collect(),
            variables: variables
                .into_iter()
                .map(|v| Variable { name: v })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::table::Table;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[test]
    fn can_create_tables() {
        let vars = vec!["a".to_string(), "b".to_string()];
        let bindings = vec![
            HashMap::from_iter(vec![("a".to_string(), "c"), ("b".to_string(), "d")].into_iter()),
            HashMap::from_iter(vec![("b".to_string(), "f"), ("a".to_string(), "e")].into_iter()),
        ];
        let table = Table::from(vars, bindings, |a| a);
        println!("Result: {:?}", table);
    }

    #[test]
    fn can_iterate() {
        let vars = vec!["a".to_string(), "b".to_string()];
        let bindings = vec![
            HashMap::from_iter(vec![("a".to_string(), "c"), ("b".to_string(), "d")].into_iter()),
            HashMap::from_iter(vec![("b".to_string(), "f"), ("a".to_string(), "e")].into_iter()),
        ];
        let table = Table::from(vars, bindings, |a| a);
        for v in &table.variables {
            print!("|{:?}", v.name);
        }
        println!("|\n|---|---|");
        for row in &table.bindings {
            for v in &table.variables {
                print!("|{:?}", row.get(&v).unwrap());
            }
            println!("|");
        }
    }
}
