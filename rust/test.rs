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
use std::path::PathBuf;
use tree_sitter_stack_graphs::ci::Tester;
use tree_sitter_stack_graphs::NoCancellation;

fn main() -> anyhow::Result<()> {
    let lc = match tree_sitter_stack_graphs_c_sharp::try_language_configuration(&NoCancellation)
    {
        Ok(lc) => lc,
        Err(err) => {
            eprintln!("{}", err.display_pretty());
            return Err(anyhow!("Language configuration error"));
        }
    };
    let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test");
    Tester::new(vec![lc], vec![test_path]).run()
}
