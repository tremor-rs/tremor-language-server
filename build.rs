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

use flate2::read::GzDecoder;
use regex::Regex;
use std::borrow::Borrow;
// used instead of halfbrown::Hashmap because bincode can't deserialize that
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use tar::Archive;
use walkdir::WalkDir;

use tremor_script::ast::FnDoc;
// TODO get rid of this once we can switch to FnDoc for aggregate functions too
use tremor_script::docs::{FunctionDoc, FunctionSignatureDoc};
use tremor_script::path::ModulePath;
use tremor_script::{registry, Script};

const LANGUAGES: &[&str] = &["tremor-script", "tremor-query"];

const TREMOR_SCRIPT_CRATE_NAME: &str = "tremor-script";
const BASE_DOCS_DIR: &str = "tremor-www-docs";

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

// TODO add ability to infer this from a build environment variable, so that we can
// override this easily for local dev testing
fn get_tremor_script_crate_path(download_dir: &str) -> String {
    let tremor_script_version = &get_cargo_lock_version_for_crate(TREMOR_SCRIPT_CRATE_NAME)
        .expect("Failed to get tremor-script version from cargo lock file");
    println!(
        "Detected tremor-script version from cargo lock file: {}",
        tremor_script_version
    );

    match get_local_cargo_registry_path_for_crate(TREMOR_SCRIPT_CRATE_NAME, tremor_script_version)
        .unwrap()
    {
        Some(path) => path,
        None => {
            println!("Could not find tremor-script src in local cargo registry, so downloading it now...");
            download_and_extract_crate(
                TREMOR_SCRIPT_CRATE_NAME,
                tremor_script_version,
                download_dir,
            )
            .unwrap()
        }
    }
}

fn parse_tremor_stdlib(tremor_script_source_dir: &str) -> HashMap<String, FunctionDoc> {
    let mut function_docs: HashMap<String, FunctionDoc> = HashMap::new();

    for entry in WalkDir::new(format!("{}/lib", tremor_script_source_dir)) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            println!("Parsing tremor file: {}", path.display());

            let module_file = File::open(Path::new(&path)).unwrap();
            let mut buffered_reader = BufReader::new(module_file);

            let mut module_text = String::new();
            buffered_reader.read_to_string(&mut module_text).unwrap();

            let module_path = ModulePath::load();
            let registry = registry::registry();

            match Script::parse(
                &module_path,
                &path.display().to_string(),
                module_text,
                &registry,
            ) {
                Ok(script) => {
                    let docs = script.docs();

                    // module name here is "self" always so can't use it right now
                    // TODO fix this?
                    //if let Some(module_doc) = &docs.module {
                    //    println!("Found module: {}", module_doc.name);
                    //}

                    // filenames match module name here
                    let module_name = path.file_stem().unwrap().to_string_lossy();
                    println!("Found module: {}", module_name);

                    for fndoc in &docs.fns {
                        let function_doc = fndoc_to_function_doc(fndoc, &module_name);
                        println!("Found function: {}", function_doc.signature);

                        function_docs
                            .insert(function_doc.signature.full_name.clone(), function_doc);
                    }
                }
                Err(e) => eprintln!("Error parsing file {}: {:?}", path.display(), e),
            }
        }
    }

    function_docs
}

fn fndoc_to_function_doc(fndoc: &FnDoc, module_name: &str) -> FunctionDoc {
    let signature_doc = FunctionSignatureDoc {
        full_name: format!("{}::{}", module_name, fndoc.name),
        args: fndoc.args.iter().map(|s| s.to_string()).collect(),
        result: String::new(), // TODO adopt comment convention to represent result type
    };

    FunctionDoc {
        signature: signature_doc,
        description: fndoc.doc.as_ref().unwrap_or(&String::new()).to_string(),
        summary: None,  // TODO add first line?
        examples: None, // TODO parse out stuff in code blocks
    }
}

fn parse_raw_function_docs(language_docs_dir: &str) -> HashMap<String, FunctionDoc> {
    let mut function_docs: HashMap<String, FunctionDoc> = HashMap::new();

    for entry in fs::read_dir(format!("{}/functions", language_docs_dir)).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // TODO figure out why this does not work
        //if path.is_file() && path.ends_with(".md") {
        if path.is_file() && path.to_str().unwrap().ends_with(".md") {
            println!("Parsing markdown file: {}", path.display());

            let module_doc_file = File::open(Path::new(&path)).unwrap();
            let mut buffered_reader = BufReader::new(module_doc_file);

            let mut module_doc_contents = String::new();
            buffered_reader
                .read_to_string(&mut module_doc_contents)
                .unwrap();

            module_doc_contents
                .split("\n### ")
                .skip(1) // first element is the module header, so skip it
                .for_each(|raw_function_doc| {
                    let function_doc = raw_doc_to_function_doc(raw_function_doc);
                    function_docs.insert(function_doc.signature.full_name.clone(), function_doc);
                });
        }
    }

    function_docs
}

fn raw_doc_to_function_doc(raw_doc: &str) -> FunctionDoc {
    let doc_parts: Vec<&str> = raw_doc.splitn(2, '\n').map(|s| s.trim()).collect();

    // TOOD use named params here
    let re = Regex::new(r"(.+)\((.*)\)\s*->(.+)").unwrap();
    let caps_raw = re.captures(doc_parts[0]);

    let signature_doc = match caps_raw {
        Some(caps) => {
            println!("Found function: {}", &caps[0]);
            FunctionSignatureDoc {
                full_name: caps[1].trim().to_string(),
                args: caps[2].split(',').map(|s| s.trim().to_string()).collect(),
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

    let function_docs = match language_name {
        "tremor-script" => {
            let tremor_script_crate_path = get_tremor_script_crate_path(dest_dir);
            println!(
                "Reading tremor script stdlib files from {}",
                tremor_script_crate_path
            );
            parse_tremor_stdlib(&tremor_script_crate_path)
        }
        _ => {
            // Tremor docs repo is needed right now for generating aggregate function documentation
            // as well as module completion items for tremor-query. Once we can can generate these
            // the same way as tremor-script, this won't be needed.
            if !Path::new(&format!("{}/LICENSE", BASE_DOCS_DIR)).exists() {
                println!("Setting up submodule dependencies...");
                run_command_or_fail(".", "git", &["submodule", "update", "--init"]);
            }
            parse_raw_function_docs(&format!("{}/docs/{}", BASE_DOCS_DIR, language_name))
        }
    };

    bincode::serialize_into(&mut f, &function_docs).unwrap();
}

// Utility functions

fn get_cargo_lock_version_for_crate(name: &str) -> Option<String> {
    let re = Regex::new(&format!(r#""{}"[\r\n]+version = "(.+)""#, name)).unwrap();
    re.captures(include_str!("Cargo.lock"))
        .map(|caps| caps[1].to_string())
}

fn get_local_cargo_registry_path_for_crate(
    name: &str,
    version: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let crate_src_glob_path = home::cargo_home()?
        .join(&format!("registry/src/*/{}-{}/", name, version))
        .display()
        .to_string();
    Ok(glob::glob(&crate_src_glob_path)?
        .map(|result| result.unwrap())
        .max()
        .map(|pathbuf| pathbuf.display().to_string()))
}

// run this async function with tokio runtime
#[tokio::main]
async fn download_and_extract_crate(
    name: &str,
    version: &str,
    dest_dir: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let crate_src_dir = format!("{}/{}-{}", dest_dir, name, version);

    if Path::new(&format!("{}/Cargo.toml", crate_src_dir)).exists() {
        println!(
            "Crate content is already there in the final source directory {}, so skipping download",
            crate_src_dir
        );
    } else {
        let download_url = format!(
            // based on https://github.com/rust-lang/crates.io/issues/1592#issuecomment-453221464
            "https://crates.io/api/v1/crates/{}/{}/download",
            name, version
        );
        println!(
            "Downloading crate `{}=={}` from {}",
            name, version, download_url
        );
        let crate_bytes = reqwest::get(&download_url).await?.bytes().await?;

        println!("Exracting crate to {}/", dest_dir);
        Archive::new(GzDecoder::new(&crate_bytes[..])).unpack(dest_dir)?;
    }

    // just follows the naming convention for a crate file (extracted above)
    Ok(format!("{}/{}-{}", dest_dir, name, version))
}

// lifted from https://github.com/fede1024/rust-rdkafka/blob/v0.23.0/rdkafka-sys/build.rs#L7
fn run_command_or_fail<P, S>(dir: &str, cmd: P, args: &[S])
where
    P: AsRef<Path>,
    S: Borrow<str> + AsRef<OsStr>,
{
    let cmd = cmd.as_ref();
    let cmd = if cmd.components().count() > 1 && cmd.is_relative() {
        // If `cmd` is a relative path (and not a bare command that should be
        // looked up in PATH), absolutize it relative to `dir`, as otherwise the
        // behavior of std::process::Command is undefined.
        // https://github.com/rust-lang/rust/issues/37868
        PathBuf::from(dir)
            .join(cmd)
            .canonicalize()
            .expect("canonicalization failed")
    } else {
        PathBuf::from(cmd)
    };
    println!(
        "Running command: \"{} {}\" in dir: {}",
        cmd.display(),
        args.join(" "),
        dir
    );
    let ret = Command::new(cmd).current_dir(dir).args(args).status();
    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => (),
        Ok((false, Some(c))) => panic!("Command failed with error code {}", c),
        Ok((false, None)) => panic!("Command got killed"),
        Err(e) => panic!("Command failed with error: {}", e),
    }
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
