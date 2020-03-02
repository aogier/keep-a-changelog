#[macro_use]
extern crate lazy_static;
extern crate clap;
use clap::{App, Arg, SubCommand};
use std::io::Write;

use regex::Regex;
//use semver::Version;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

const KAC_HEADER: &str = "# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).";

const LIST_ITEM: &str = "-";

#[derive(Serialize, Deserialize, Debug, Default)]
struct Changelog {
    releases: Vec<Release>,
    configuration: Configuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Release {
    version: String,
    release_date: String,
    sections: Vec<Section>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Section {
    category: String,
    lines: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Configuration {
    output_format: String,
    git_provider: String,
    repo_name: String,
    tag_template: String,
}

impl Changelog {
    fn new() -> Self {
        Self::default()
    }
}

trait MarkdownSerializable {
    fn to_markdown(&self) -> String;
}

impl MarkdownSerializable for Configuration {
    fn to_markdown(&self) -> String {
        let mut ret = String::from("[//]: # (C3-1");
        if self.output_format != "" {
            ret = format!("{}-D{}", ret, self.output_format)
        }
        if self.git_provider != "" {
            ret = format!("{}-G{}", ret, self.git_provider)
        }
        if self.repo_name != "" {
            ret = format!("{}-R{}", ret, self.repo_name)
        }
        if self.tag_template != "" {
            ret = format!("{}-T{}", ret, self.tag_template)
        }

        format!("{})", ret)
    }
}

impl MarkdownSerializable for Section {
    fn to_markdown(&self) -> String {
        let mut ret = format!("### {}\n", self.category);

        for line in self.lines.iter() {
            ret = format!("{}\n{} {}", ret, LIST_ITEM, line)
        }

        format!("{}\n", ret)
    }
}

impl MarkdownSerializable for Release {
    fn to_markdown(&self) -> String {
        let mut ret = format!("## [{}]", self.version);
        if self.release_date != "" {
            ret = format!("{} - {}", ret, self.release_date)
        }
        ret = format!("{}\n", ret);

        for section in self.sections.iter() {
            ret = format!("{}\n{}", ret, section.to_markdown())
        }

        ret
    }
}

impl MarkdownSerializable for Changelog {
    fn to_markdown(&self) -> String {
        let mut ret = format!("{}\n", KAC_HEADER);

        for release in self.releases.iter() {
            ret = format!("{}\n{}", ret, release.to_markdown())
        }

        format!("{}\n{}", ret, self.configuration.to_markdown())
    }
}

fn add_release(changelog: &mut Changelog, line: String) {
    lazy_static! {
        static ref RELEASE_PATTERN: Regex =
            Regex::new(r"^## \[(?P<version>.*)\](?: - (?P<date>.*))?$").unwrap();
    }

    let rel = RELEASE_PATTERN.captures(&line).unwrap();

    let release = Release {
        version: rel
            .name("version")
            .map_or("".to_string(), |m| m.as_str().to_string()),

        release_date: rel
            .name("date")
            .map_or("".to_string(), |m| m.as_str().to_string()),

        ..Default::default()
    };

    changelog.releases.push(release)
}

fn add_category(changelog: &mut Changelog, line: String) {
    lazy_static! {
        static ref CATEGORY_PATTERN: Regex = Regex::new(r"^### (?P<category>.*)$").unwrap();
    }

    let cat = CATEGORY_PATTERN.captures(&line).unwrap();
    let section = Section {
        category: cat
            .name("category")
            .map_or("".to_string(), |m| m.as_str().to_string()),

        ..Default::default()
    };

    if let Some(release) = changelog.releases.last_mut() {
        release.sections.push(section);
    }
}

fn add_change(changelog: &mut Changelog, line: String) {
    lazy_static! {
        static ref CHANGE_PATTERN: Regex = Regex::new(r"^- (?P<line>.*)$").unwrap();
    }

    let line = CHANGE_PATTERN.captures(&line).unwrap();

    let entry = line
        .name("line")
        .map_or("".to_string(), |m| m.as_str().to_string());

    if let Some(release) = changelog.releases.last_mut() {
        if let Some(section) = release.sections.last_mut() {
            section.lines.push(entry);
        }
    }
}

fn add_config(changelog: &mut Changelog, line: String) {
    lazy_static! {
        static ref CONFIG_PATTERN: Regex =
            Regex::new(r"^\[//\]: # \(C3-1-(?P<config>.*)\)$").unwrap();
    }

    let line = CONFIG_PATTERN.captures(&line).unwrap();

    let config = line
        .name("config")
        .map_or("".to_string(), |m| m.as_str().to_string());

    let sections = config.split('-');
    let mut configuration: Configuration = Default::default();
    for section in sections {
        let (section, value) = section.split_at(1);
        match section {
            "D" => configuration.output_format = value.to_string(),
            "G" => configuration.git_provider = value.to_string(),
            "R" => configuration.repo_name = value.to_string(),
            "T" => configuration.tag_template = value.to_string(),
            _ => println!("Undefined configuration option: {:?}", value),
        }
    }

    changelog.configuration = configuration;
}

fn main() {
    let matches = App::new("CHACHACHA")
        .about("\nDoes awesome things")
        .author("Alessandro -oggei- Ogier <alessandro.ogier@gmail.com>")
        .arg(
            Arg::with_name("filename")
                .short("f")
                .long("filename")
                .value_name("FILE")
                .help("Sets changelog's path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("added")
                .about("Add an 'added' entry")
                .arg(Arg::with_name("line").help("Line to add").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("changed")
                .about("Add a 'changed' entry")
                .arg(Arg::with_name("line").help("Line to add").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("deprecated")
                .about("Add a 'deprecated' entry")
                .arg(Arg::with_name("line").help("Line to add").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("fixed")
                .about("Add a 'fixed' entry")
                .arg(Arg::with_name("line").help("Line to add").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("removed")
                .about("Add a 'removed' entry")
                .arg(Arg::with_name("line").help("Line to add").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("security")
                .about("Add a 'security' entry")
                .arg(Arg::with_name("line").help("Line to add").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("initialize a new file")
                .arg(
                    Arg::with_name("overwrite")
                        .long("overwrite")
                        .help("Overwrite an existing file"),
                ),
        )
        .get_matches();

    let filename = matches.value_of("filename").unwrap_or("CHANGELOG.md");

    if let Some(matches) = matches.subcommand_matches("init") {
        let path = Path::new(filename);
        if path.exists() {
            if matches.args.contains_key("overwrite") {
                println!("let's overwrite!");
                {
                    let mut f = File::create(filename).unwrap();

                    f.write(b"ciao");

                    f.sync_all();
                }
            } else {
                let error = clap::Error {
                    message: String::from("File exists and no overwrite flag is set."),
                    info: None,
                    kind: clap::ErrorKind::ValueValidation,
                };
                clap::Error::exit(&error);
            }
        };

        if matches.is_present("init") {
            println!("initializing");
            std::process::exit(0);
        } else {
            println!("boh: {:#?}", matches.args["overwrite"]);
            std::process::exit(0);
        }
    };

    println!("{:?}", matches);

    let mut changelog = Changelog::new();

    let release_pattern = Regex::new(r"^## \[.*\]( - .*)?$").unwrap();
    let category_pattern = Regex::new(r"^### .*$").unwrap();
    let config_pattern = Regex::new(r"^\[//\]: # .*$").unwrap();
    let nonlink_pattern = Regex::new(r"^- .*$").unwrap();

    if let Ok(lines) = read_lines(filename) {
        for raw_line in lines {
            if let Ok(line) = raw_line {
                if release_pattern.is_match(&line) {
                    add_release(&mut changelog, line);
                } else if category_pattern.is_match(&line) {
                    add_category(&mut changelog, line);
                } else if config_pattern.is_match(&line) {
                    add_config(&mut changelog, line);
                } else if nonlink_pattern.is_match(&line) {
                    add_change(&mut changelog, line);
                }
            }
        }
    } else {
        println!("PORCODIO");
    }

    println!("{}", changelog.to_markdown());

    let j = serde_json::to_string(&changelog).unwrap();
    println!("{}", j);
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
