use std::path::PathBuf;
use anyhow::Error;
use anyhow::Ok;
use clap::Args;
use clap::Parser;
use clap;
use stack_graphs::storage::SQLiteReader;
use tree_sitter_stack_graphs::cli::database::DatabaseArgs;

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
    
    #[clap(long, short = 't', required = true)]
    pub node_type: String,
}

impl FindNodeArgs {
    pub fn run(self, db_path: &PathBuf) -> anyhow::Result<()>{
        println!("db_path {db_path:?} -- type {self:?}");
        let mut db = SQLiteReader::open(&db_path)?;

        let paths = Self::get_file_strings(&mut db)?;

        for path in paths {
            let _ = db.load_graph_for_file(path.as_str())?;
        }

        let (graph, _, _) = db.get();
        for handle in graph.iter_nodes() {
            println!("node {handle:?}");
            let node = &graph[handle];
            println!("node {}", node.display(graph));
            let edges = graph.outgoing_edges(handle);
            let count = edges.count();
            println!("edges: {}", count)
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
}