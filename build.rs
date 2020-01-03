// Copyright 2018-2020, Wayfair GmbH
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

use bincode;
use regex::Regex;
// used instead of halfbrown::Hashmap because bincode can't deserialize that
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;
use std::process;
use tremor_script::docs::{FunctionDoc, FunctionSignatureDoc};

const LANGUAGES: &[&str] = &["tremor-script", "tremor-query"];

const BASE_DOCS_DIR: &str = "../tremor-runtime/docs";

/*
fn get_test_function_doc(language_name: &str) -> (String, FunctionDoc) {
    let test_func = match language_name {
        "tremor-script" => "random::bool".to_string(),
        "tremor-query" => "stats::min".to_string(),
        _ => unreachable!(),
    };

    let test_doc = match language_name {
        "tremor-script" => FunctionDoc {
            signature: "random::bool() -> bool".to_string(),
            description: "Generates a random boolean.".to_string(),
            summary: None,
            examples: Some("```random::bool() # either true or false```".to_string()),
        },
        "tremor-query" => FunctionDoc {
            signature: "stats::min(int|float) -> int|float".to_string(),
            description: "Determines the smallest event value in the current windowed operation."
                .to_string(),
            summary: None,
            examples: Some("```trickle\nstats::min(event.value)\n```".to_string()),
        },
        _ => unreachable!(),
    };

    (test_func, test_doc)
}
*/

fn parse_raw_function_docs(language_name: &str) -> HashMap<String, FunctionDoc> {
    let mut function_docs: HashMap<String, FunctionDoc> = HashMap::new();

    let function_docs_path = Path::new(BASE_DOCS_DIR)
        .join(language_name)
        .join("functions");

    for entry in fs::read_dir(&function_docs_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        //println!("{:?}", path.to_str());
        //dbg!(path.to_str().unwrap().ends_with(".md"));
        // TODO figure out why this does not work
        //dbg!(path.ends_with("md"));
        //if path.is_file() && path.ends_with(".md") {
        if path.is_file() && path.to_str().unwrap().ends_with(".md") {
            println!("Parsing markdown file: {:?}", path);

            let module_doc_file = File::open(Path::new(&path)).unwrap();
            //File::open(Path::new(&function_docs_path).join(module_doc_filename)).unwrap();
            //.map_err(|e| Error::from(format!("Could not open file {} => {}", file_name, e)))?;
            let mut buffered_reader = BufReader::new(module_doc_file);

            let mut module_doc_contents = String::new();
            buffered_reader.read_to_string(&mut module_doc_contents);

            // test
            // TODO remove
            //let (test_func, mut test_doc) = get_test_function_doc(language_name);
            //function_docs.insert(test_func, test_doc);

            module_doc_contents
                .split("\n### ")
                .skip(1) // first element is the module header, so skip it
                .for_each(|raw_function_doc| {
                    let function_doc = to_function_doc(raw_function_doc);
                    function_docs.insert(function_doc.signature.full_name.clone(), function_doc);
                });
        }
    }

    function_docs
}

fn to_function_doc(raw_doc: &str) -> FunctionDoc {
    let doc_parts: Vec<&str> = raw_doc.splitn(2, '\n').map(|s| s.trim()).collect();

    // TOOD use named params here
    let re = Regex::new(r"(.+)\((.*)\)\s*->(.+)").unwrap();
    let caps_raw = re.captures(doc_parts[0]);

    let signature_doc = match caps_raw {
        Some(caps) => {
            println!("Found function: {}", &caps[0]);
            FunctionSignatureDoc {
                full_name: caps[1].trim().to_string(),
                args: caps[2].split(",").map(|s| s.trim().to_string()).collect(),
                result: caps[3].trim().to_string(),
            }
        }
        None => {
            // TODO proper error handling here
            dbg!(&doc_parts[0]);
            dbg!(&caps_raw);
            FunctionSignatureDoc {
                full_name: "random::string".to_string(),
                args: vec!["length".to_string()],
                result: "string".to_string(),
            }
        }
    };
    FunctionDoc {
        signature: signature_doc,
        description: doc_parts[1].to_string(),
        summary: None,  // TODO add first line?
        examples: None, // TODO parse out stuff in code blocks
    }
}

// TODO remove unwraps here
fn bindump_function_docs(language_name: &str, dest_dir: &str) {
    let dest_path = Path::new(dest_dir).join(format!("function_docs.{}.bin", language_name));
    let mut f = BufWriter::new(File::create(&dest_path).unwrap());

    println!(
        "Dumping function docs for {} to: {}",
        language_name,
        dest_path.to_str().unwrap()
    );
    bincode::serialize_into(&mut f, &parse_raw_function_docs(language_name)).unwrap();
}

fn main() {
    match env::var("OUT_DIR") {
        Ok(out_dir) => {
            for language_name in LANGUAGES {
                bindump_function_docs(language_name, &out_dir);
            }
        }
        Err(e) => {
            eprintln!("Variable: {}\nError: {}", "OUT_DIR", e);
            process::exit(1)
        }
    }
}
