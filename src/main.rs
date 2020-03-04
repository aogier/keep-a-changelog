use clap::{App, Arg, SubCommand};
use pulldown_cmark::{html, CowStr, Event, InlineStr, Options, Parser, Tag};

use std::fs;
use std::io::Write;

#[macro_use]
extern crate lazy_static;
use regex::Regex;

use std::collections::VecDeque;

mod markdown;

const HEADING_RELEASE: u32 = 2;
const HEADING_SECTION: u32 = 3;

struct Changelog<'c> {
    parser: Parser<'c>,
    buffer: VecDeque<Event<'c>>,
    options: &'c CliOptions,
    fields_order: Vec<&'c str>,
    done_section: bool,
    // rendered_option: String,
    string_buffer: String,
}

// impl Changelog{
//     fn parse() -> &'static str{
//         return "ciao"
//     }
// }

impl<'c> Changelog<'c> {
    fn new(markdown_input: &'c String, options: &'c CliOptions) -> Changelog<'c> {
        let parser = Parser::new_with_broken_link_callback(
            &markdown_input,
            Options::empty(),
            Some(&|norm, raw| None),
        );
        Changelog {
            parser,
            buffer: VecDeque::with_capacity(4),
            options,
            fields_order: vec![
                "Added",
                "Changed",
                "Deprecated",
                "Removed",
                "Fixed",
                "Security",
            ],
            done_section: false,
            string_buffer: String::from(""),
        }
    }

    fn enqueue_item(&mut self, item: &'c String) {
        self.buffer.push_front(Event::Start(Tag::Item));
        self.buffer.push_front(Event::Text(CowStr::Borrowed(item)));
        self.buffer.push_front(Event::End(Tag::Item));
    }

    fn pushback_item(&mut self, item: &'c str) {
        self.buffer.push_back(Event::End(Tag::Item));
        self.buffer.push_back(Event::Text(CowStr::Borrowed(item)));
        self.buffer.push_back(Event::Start(Tag::Item));
    }

    fn pushback_heading(&mut self, item: &'c str, heading: u32) {
        self.buffer.push_back(Event::End(Tag::Heading(heading)));
        self.buffer.push_back(Event::Text(CowStr::Borrowed(item)));
        self.buffer.push_back(Event::Start(Tag::Heading(heading)));
    }
}

impl<'c> Iterator for Changelog<'c> {
    type Item = Event<'c>;

    fn next(&mut self) -> Option<Event<'c>> {
        lazy_static! {
            static ref RELEASE_PATTERN: Regex =
                Regex::new(r"^\[(?P<version>.*)\](?: +- +(?P<date>.*) *)?$").unwrap();
        }

        if !self.buffer.is_empty() {
            let next = self.buffer.pop_back();
            return next;
        }

        let next = self.parser.next();

        match next {
            Some(Event::Start(Tag::Heading(2))) if !self.done_section => {
                // we are in the first release heading, is it
                // "Unreleased" already?

                let release = self.parser.next();

                match &release {
                    Some(Event::Text(text)) => {
                        // a text-only / broken link release specification
                        // consume iterator until heading's end

                        let mut release_vec = Vec::with_capacity(4);

                        release_vec.push(text.to_owned());
                        self.buffer.push_front(release.unwrap());

                        loop {
                            let release = self.parser.next();
                            match &release {
                                Some(Event::Text(text)) => {
                                    release_vec.push(text.clone());
                                    self.buffer.push_front(release.unwrap());
                                }
                                _ => {
                                    self.buffer.push_front(release.unwrap());
                                    let raw_release = release_vec.join("");
                                    let parsed_release = RELEASE_PATTERN.captures(&raw_release);
                                    match parsed_release {
                                        Some(x) => {
                                            match x.name("version") {
                                                Some(x) => {
                                                    let release_string = x.as_str();
                                                    // println!("version: <{}>", release_string);
                                                    if release_string.to_ascii_lowercase()
                                                        != "unreleased"
                                                    {
                                                        // println!("non unre");
                                                        self.buffer.push_back(Event::Start(
                                                            Tag::Heading(HEADING_RELEASE),
                                                        ));

                                                        self.buffer
                                                            .push_back(Event::End(Tag::List(None)));

                                                        self.pushback_item(&self.options.argument);

                                                        self.buffer.push_back(Event::Start(
                                                            Tag::List(None),
                                                        ));

                                                        self.pushback_heading(
                                                            self.fields_order[self
                                                                .fields_order
                                                                .iter()
                                                                .position(|&x| {
                                                                    x.eq_ignore_ascii_case(
                                                                        &self.options.action,
                                                                    )
                                                                })
                                                                .unwrap()],
                                                            HEADING_SECTION,
                                                        );

                                                        self.buffer.push_back(Event::End(
                                                            Tag::Heading(HEADING_RELEASE),
                                                        ));
                                                        self.buffer.push_back(Event::Text(
                                                            CowStr::Borrowed("[Unreleased]"),
                                                        ));
                                                    }
                                                }
                                                None => {
                                                    println!(
                                                        "cannot parse version, bailing out..."
                                                    );
                                                }
                                            }

                                            // if x.
                                        }
                                        None => println!("malformed header: {}", raw_release),
                                    }
                                    break;
                                }
                            }
                        }

                        // println!("BROKEN LINK {}", &release_vec.join(""));
                    }
                    _ => (),
                }

                // print!("\nAIEEI {:?}", release);
                // let release = self.parser.next();
                // print!("\nAIEEI {:?}", release);
                // let release = self.parser.next();
                // print!("\nAIEEI {:?}", release);
                // let release = self.parser.next();
                // print!("\nAIEEI {:?}", release);
                // let release = self.parser.next();
                // print!("\nAIEEI {:?}", release);

                self.done_section = true;
            }
            _ => (),
        }

        // if let Some(Event::Start(Tag::Heading(3))) = &next {
        //     let title_event = self.parser.next().expect("Error getting heading title");

        //     if let Event::Text(header_title) = &title_event {
        //         // calculate
        //         let position_in_order = self
        //             .fields_order
        //             .iter()
        //             .position(|&x| x == header_title.to_string());

        //         let command_position = self
        //             .fields_order
        //             .iter()
        //             .position(|&x| x.eq_ignore_ascii_case(&self.options.action));

        //         if !self.done_section && position_in_order == command_position {
        //             //pop a couple items from iterator
        //             let end_of_heading = self.parser.next().unwrap();
        //             let start_of_list = self.parser.next().unwrap();

        //             self.buffer.push_front(next.unwrap());
        //             self.buffer.push_front(title_event);
        //             // end of heading
        //             self.buffer.push_front(end_of_heading);

        //             self.buffer.push_front(start_of_list);

        //             self.enqueue_item(&self.options.argument);
        //             // self.buffer.push_front(Event::Start(Tag::Item));
        //             // self.buffer
        //             //     .push_front(Event::Text(CowStr::Borrowed(&self.options.argument)));
        //             // self.buffer.push_front(Event::End(Tag::Item));

        //             self.done_section = true;
        //             return self.next();
        //         }
        //     // else if position_in_order > command_position && !self.done_section {
        //     //     // println!("baugigi");

        //     //     self.buffer.push(title_event);
        //     //     self.buffer.push(next.unwrap());
        //     //     self.buffer.push(Event::End(Tag::List(None)));
        //     //     self.buffer.push(Event::End(Tag::Item));
        //     //     self.buffer
        //     //         .push(Event::Text(CowStr::Borrowed(&self.options.argument)));
        //     //     self.buffer.push(Event::Start(Tag::Item));
        //     //     self.buffer.push(Event::Start(Tag::List(None)));
        //     //     self.buffer.push(Event::End(Tag::Heading(3)));
        //     //     self.buffer.push(Event::Text(CowStr::Borrowed(
        //     //         &self.fields_order[command_position.unwrap()],
        //     //     )));
        //     //     self.buffer.push(Event::Start(Tag::Heading(3)));
        //     //     self.done_section = true;
        //     //     return self.next();
        //     // }

        //     // println!(
        //     //     "[XXX] - following: {} (pos {:?}, req: {:?})",
        //     //     header_title, position_in_order, command_position
        //     // );
        //     } else {
        //         println!("bauscia");
        //     }

        //     self.buffer.push_front(next.unwrap());
        //     self.buffer.push_front(title_event);

        //     // self.buffer.push(Event::Start(Tag::Heading(*xx)));
        //     // self.buffer.push(Event::End(Tag::Heading(*xx)));
        //     // self.buffer
        //     //     .push(Event::Text(CowStr::Borrowed("[culocane]")));
        //     // self.buffer.push(Event::Start(Tag::Heading(*xx)));
        //     return self.next();
        // };

        next
    }
}

struct CliOptions {
    action: String,
    argument: String,
}

fn main() {
    let matches = App::new("CHACHACHA")
        .about("\nDoes awesome things")
        .version("0-muku")
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

    let (subcommand, smatches) = matches.subcommand();

    let mut diocane = String::from("");
    for merda in smatches.unwrap().values_of("line").unwrap() {
        diocane.push_str(&format!(" {}", &merda));
    }

    let markdown_input =
        fs::read_to_string("CHANGELOG.md").expect("Something went wrong reading the file");

    let options = CliOptions {
        action: String::from(subcommand),
        argument: String::from(diocane.trim()),
    };

    let changelog = Changelog::new(&markdown_input, &options);

    // Write to anything implementing the `Write` trait. This could also be a file
    // or network socket.
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(b"\nHTML output:\n").unwrap();

    // html::write_html(&mut handle, changelog).unwrap();
    markdown::write(&mut handle, changelog).unwrap();
}
