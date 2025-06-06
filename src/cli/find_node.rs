use std::path::PathBuf;
use std::vec;
use anyhow::Error;
use anyhow::Ok;
use clap::Args;
use clap::Parser;
use clap;
use stack_graphs::graph::Degree;
use stack_graphs::graph::NodeID;
use stack_graphs::graph::StackGraph;
use stack_graphs::storage::SQLiteReader;
use tree_sitter_stack_graphs::cli::database::DatabaseArgs;
use regex::Regex;

#[derive(Parser)]
pub struct FindNode {
    #[clap(flatten)]
    db_args: DatabaseArgs,
    #[clap(flatten)]
    find_node_args: FindNodeArgs,
}

impl FindNode{
    pub fn run(self, default_db_path: PathBuf) -> anyhow::Result<()> {
        let db_path = self.db_args.get_or(default_db_path);
        return self.find_node_args.run(&db_path);
    }
}

#[derive(Args)]
#[derive(Debug)]
pub struct FindNodeArgs {
    
    #[clap(long, short = 't')]
    pub node_type: String,
    #[clap(long, short = 'r', required = true)]
    pub regex: String,
}

impl FindNodeArgs {
    pub fn run(self, db_path: &PathBuf) -> anyhow::Result<()>{
        println!("db_path {db_path:?} -- type {self:?}");
        let mut db = SQLiteReader::open(&db_path)?;

        let paths = Self::get_file_strings(&mut db)?;

        for path in paths {
            println!("loading path {path}");
            let _ = db.load_graph_for_file(path.as_str())?;
        }
        let (graph, _, _) = db.get();

        // Now that everything is loaded, we will need to determine what we are searching for.
        let regex_split: Vec<&str> = self.regex.split(".").collect();

        let mut found_nodes: Vec<NodeID> = vec![];
        for handle in graph.iter_nodes() {
            let node = &graph[handle];
            let mut should_traverse = false;
            match node.symbol() {
                Some(symbol_handle) => {
                    let symbol = &graph[symbol_handle];
                    if Self::does_symbol_match_regex(self.regex.as_str(), symbol) {
                        if node.is_reference() {
                            println!("found node {}", node.display(graph));
                            found_nodes.push(node.id());
                        }
                    }
                    // This means that the search is not a FQDN, at this point
                    // the iter at this level needs to find the searched string
                    if !self.regex.contains(".") {
                        continue;
                    }
                    should_traverse = graph.outgoing_edges(handle).any(|edge| {
                        let next_node = &graph[edge.sink];
                        match next_node.symbol() {
                            Some(symbol_handle) => {
                                let next_symbol = &graph[symbol_handle];
                                if next_symbol == "." {
                                    // Lets determine if this node is matching the start of the ask
                                    return true
                                }
                                return false;
                            }
                            None => {
                                return false
                            }
                        }
                    }) && graph.incoming_edge_degree(handle) == Degree::Zero;
                }
                None => {}
            }
            if should_traverse {
                println!("traversing node - {}", node.display(graph));
                let node_id = node.id();
                let _ = node;
                let return_ids = Self::traverse_tree(regex_split.clone(), graph, node_id);
                match return_ids {
                    Some(ids) => {
                        found_nodes.extend(ids);
                        
                    }
                    None => {}
                }
            }
        }
        println!("Found Nodes");
        for id in found_nodes {
            match graph.node_for_id(id) {
                Some(h) => {
                    let n = &graph[h];
                    println!("Node: {} matches", n.display(graph))
                }
                None => {
                    println!("some went wrong unable to find node for id: {:?}", id)
                }
            }

        }
        Ok(())
    }

    fn get_file_strings(db: &mut SQLiteReader) -> anyhow::Result<Vec<String>, Error>{
        let mut file_strings: Vec<String>  = vec![];
        let mut files = db.list_all()?;
        for file in files.try_iter()?{
            let entry = file?;
            let file_path= entry.path.into_os_string().into_string().unwrap();
            file_strings.push(file_path);
        }
        return Ok(file_strings)
    }

    fn does_symbol_match_regex(regex: &str, symbol: &str) -> bool {
        if regex == "*" {
            return true
        }
        if regex.contains("*") {
            
            let r = Regex::new(regex).unwrap();
            return r.is_match(symbol);
        }
        if regex == symbol {
            return true;
        }
        return false;
    }

    fn traverse_tree(regex_split: Vec<&str>, graph: &mut StackGraph, node_id: NodeID) -> Option<Vec<NodeID>> {
        let mut should_traverse_nodes: Vec<NodeID> = vec![];
        let node_handle = graph.node_for_id(node_id)?;

        let node = &graph[node_handle];
        let symbol_handle= node.symbol()?;
        let symbol = &graph[symbol_handle];
        let symbol_parts:Vec<&str> = symbol.split(".").collect();
        let mut parts_not_matched = false;
        for (i, partial) in symbol_parts.iter().enumerate() {
            if i >= regex_split.len() {
                parts_not_matched = true;
                break;
            }
            if !Self::does_symbol_match_regex(regex_split[i], partial) {
                parts_not_matched = true;
            }
        }
        if parts_not_matched {
            return None
        }
        if symbol_parts.len() == regex_split.len() {
            return Some(vec![node_id]);
        }
        let mut new_regex_split = regex_split.clone();
        new_regex_split.clear();
        for (i, s) in regex_split.iter().enumerate() {
            if i < symbol_parts.len() {
                continue;
            }
            new_regex_split.push(s);
        }

        println!("new regex split {new_regex_split:?}");
        
        let mut return_ids: Vec<NodeID> = vec![];
        // We need to handle the case, where the symbol parts match the regex_parts and we shouldn't continue to pop.
        for edge in graph.outgoing_edges(node_handle) {
            let node =  &graph[edge.sink];
            match node.symbol() {
                Some(symbol_handle) => {
                    let symbol  = &graph[symbol_handle];
                    if symbol == "." {
                        continue;
                    }
                    let symbol_split: Vec<&str> = symbol.split(".").collect();
                    let mut not_found = false;
                    for (i, partial) in symbol_split.iter().enumerate() {
                        if i >= new_regex_split.len() {
                            not_found = true;
                            break;
                        }
                        if !Self::does_symbol_match_regex(new_regex_split[i], partial) {
                            not_found = true;
                            break;
                        }
                    }
                    if !not_found && symbol_split.len() == new_regex_split.len(){
                        println!("found {} -- {} -- {}", node.display(graph), symbol, node_id.display(graph));
                        return_ids.push(node.id());
                    } else if !not_found && symbol_split.len() != new_regex_split.len(){
                        println!("should continue traverse {} -- {} -- {}", node.display(graph), symbol, node_id.display(graph));
                        should_traverse_nodes.push(node.id())
                    }
                }
                None => { continue;}
            }
        }
        if !should_traverse_nodes.is_empty() {
            for node_id in should_traverse_nodes {
                let x: Option<Vec<NodeID>> = Self::traverse_tree(new_regex_split.clone(), graph, node_id);
                match x {
                    Some(x) => return_ids.extend(&x),
                    None => {}
                }
            }
        }
        return Some(return_ids);
    }
}