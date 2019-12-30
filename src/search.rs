use pdf::*;
use object::*;
use backend::*;
use primitive::*;

use std::collections::{VecDeque, HashSet};
use std::collections::vec_deque;
use std::str::FromStr;

use Inspector;


#[derive(Debug)]
pub enum PathElem {
    DictElem {key: String},
    ArrayElem {index: i32},
}
impl PathElem {
    pub fn from_dict_key(key: &str) -> PathElem {
        PathElem::DictElem {key: String::from_str(key).unwrap()}
    }
    pub fn from_array_index(index: i32) -> PathElem {
        PathElem::ArrayElem {index: index}
    }
}

#[derive(Default, Debug)]
pub struct SearchPath {
    path: VecDeque<PathElem>,
}
impl SearchPath {
    pub fn new(start: PathElem) -> SearchPath {
        let mut new_path = SearchPath::default();
        new_path.path.push_back(start);
        new_path
    }
    /// Add element to the left-hand side, or the beginning, of the path
    pub fn add_to_start(&mut self, elem: PathElem) {
        self.path.push_front(elem);
    }
    pub fn iter(&self) -> vec_deque::Iter<PathElem> {
        self.path.iter()
    }
}


/// Used internally to store information about where we have been
struct SearchAlg<'a, R: Resolve + 'a> {
    // Only follow each reference once
    blacklist: HashSet<PlainRef>,
    resolve: &'a R,
}
impl<'a, R: Resolve> SearchAlg<'a, R> {
    fn new(r: &'a R) -> SearchAlg<'a, R> {
        SearchAlg {
            blacklist: HashSet::new(),
            resolve: r
        }
    }
    fn search_key(&mut self, node: &Primitive, search_key: &str) -> Vec<SearchPath> {
        match *node {
            Primitive::Stream (ref stream) => {
                self.search_key(&stream.info.clone().into(), search_key)
            },
            Primitive::Dictionary (ref dict) => {
                let mut result = Vec::new();
                for (key, node) in dict.iter() {
                    if key == search_key {
                        result.push( SearchPath::new(PathElem::from_dict_key(key)) );
                    }


                    // Traverse the child nodes, adding this node the partial paths that result
                    let mut subresults = self.search_key(node, search_key);
                    for path in &mut subresults {
                        path.add_to_start(PathElem::from_dict_key(key));
                    }
                    result.append(&mut subresults);
                }
                result
            },
            Primitive::Array (ref arr) => {
                let mut result = Vec::new();
                for node in arr {
                    let mut subresults = self.search_key(node, search_key);
                    for (i, path) in &mut subresults.iter_mut().enumerate() {
                        path.add_to_start(PathElem::from_array_index(i as i32));
                    }
                    result.append(&mut subresults);
                }
                result
            },
            Primitive::Reference (reference) => {
                if !self.blacklist.contains(&reference) {
                    self.blacklist.insert(reference);
                    let prim = match self.resolve.resolve(reference) {
                        Ok(prim) => prim,
                        Err(_) => return Vec::new(),
                    };
                    self.search_key(&prim, search_key)
                } else {
                    Vec::new()
                }
            },
            _ => Vec::new(),
        }
    }
}

impl<'a, 'b, R: Resolve> Inspector<'a, 'b, R> {

    pub fn search_key(&self, node: &Primitive, search_key: &str) -> Vec<SearchPath> {
        let mut alg = SearchAlg::new(self.resolve);
        alg.search_key(node, search_key)
    }
}
