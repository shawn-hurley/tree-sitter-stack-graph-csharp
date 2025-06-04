// Copyright 2025 shawn@hurley.page
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::anyhow;
use clap::Parser;
use clap::Subcommand;
use tree_sitter_stack_graphs::cli::provided_languages::Clean;
use tree_sitter_stack_graphs::cli::provided_languages::Visualize;
use tree_sitter_stack_graphs::cli::provided_languages::Index;
use tree_sitter_stack_graphs::cli::provided_languages::Status;
use tree_sitter_stack_graphs::loader::LanguageConfiguration;
use std::path::PathBuf;
use tree_sitter_stack_graphs::cli::database::default_user_database_path_for_crate;
use tree_sitter_stack_graphs::NoCancellation;

use tree_sitter_stack_graphs_c_sharp::cli::find_node::FindNode;

fn main() -> anyhow::Result<()> {
    let lc = match tree_sitter_stack_graphs_c_sharp::try_language_configuration(&NoCancellation)
    {
        Ok(lc) => lc,
        Err(err) => {
            eprintln!("{}", err.display_pretty());
            return Err(anyhow!("Language configuration error"));
        }
    };
    let cli = Cli::parse();
    let default_db_path = default_user_database_path_for_crate(env!("CARGO_PKG_NAME"))?;
    cli.subcommand.run(default_db_path, vec![lc])
}

#[derive(Parser)]
#[clap(about, version)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: ExtendedSubcommands,
}

#[derive(Subcommand)]
pub enum ExtendedSubcommands{
    Clean(Clean),
    Index(Index),
    Status(Status),
    Visualize(Visualize),
    FindNode(FindNode)
}

impl ExtendedSubcommands {
    pub fn run(self, default_db_path: PathBuf, config: Vec<LanguageConfiguration>) -> anyhow::Result<()> {
        match self {
            Self::Clean(cmd) => cmd.run(default_db_path),
            Self::Index(cmd) => cmd.run(default_db_path, config),
            Self::Status(cmd) => cmd.run(default_db_path),
            Self::Visualize(cmd) => cmd.run(default_db_path),
            Self::FindNode(cmd) => cmd.run(default_db_path),
        }
    }
}