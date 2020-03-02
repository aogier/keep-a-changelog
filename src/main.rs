use pulldown_cmark::{Event, Options, Parser, Tag};
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Changelog {
    header: String,
    description: String,
    children: Vec<Release>,
    configuration: Configuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Release {
    header: String,
    description: String,
    children: Vec<Section>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Section {
    header: String,
    children: Vec<Change>,
}
#[derive(Serialize, Deserialize, Debug, Default)]
struct Change {
    text: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Configuration {
    output_format: String,
    git_provider: String,
    repo_name: String,
    tag_template: String,
}

// #[derive(Deserialize, Serialize, Debug, Default)]
// struct VersionInfo {
//     version: String,
//     release_date: String,
// }

impl Changelog {
    fn new() -> Self {
        Self::default()
    }

    fn parse<P>(&mut self, filename: P) -> Result<&str, io::Error>
    where
        P: AsRef<Path>,
    {
        let raw = fs::read_to_string(filename)?;
        let parser = Parser::new_ext(&raw, Options::empty());
        let mut current_buffer = &mut self.header;

        for event in parser {
            match event {
                Event::Start(event) => match event {
                    Tag::Heading(heading) => match heading {
                        1 => current_buffer = &mut self.header,
                        2 => {
                            let release = Release::default();
                            self.children.push(release);
                            current_buffer = &mut self.children.last_mut().unwrap().header;
                        }
                        3 => {
                            let section = Section::default();
                            self.children.last_mut().unwrap().children.push(section);
                            current_buffer = &mut self
                                .children
                                .last_mut()
                                .unwrap()
                                .children
                                .last_mut()
                                .unwrap()
                                .header;
                        }
                        _ => (),
                    },
                    Tag::Link(_shortcut, _borrowed, _foo) => current_buffer.push_str("["),
                    Tag::Item => {
                        let entry = Change::default();

                        self.children
                            .last_mut()
                            .unwrap()
                            .children
                            .last_mut()
                            .unwrap()
                            .children
                            .push(entry);

                        current_buffer = &mut self
                            .children
                            .last_mut()
                            .unwrap()
                            .children
                            .last_mut()
                            .unwrap()
                            .children
                            .last_mut()
                            .unwrap()
                            .text;
                    }
                    Tag::Paragraph => current_buffer.push_str("\n\n"),
                    _ => println!("MISSING START EVENT: {:?}", event),
                },

                Event::End(event) => match event {
                    Tag::Link(_shortcut, borrowed, _foo) => {
                        current_buffer.push_str(&format!("]({})", &borrowed));
                    }
                    Tag::Heading(heading) => match heading {
                        1 => current_buffer = &mut self.description,
                        2 => current_buffer = &mut self.children.last_mut().unwrap().description,
                        _ => (),
                    },
                    Tag::Paragraph => (),
                    _ => println!("TODO end event: {:?}", event),
                },

                Event::Text(text) => current_buffer.push_str(&text),
                Event::SoftBreak => current_buffer.push_str(" "),
                _ => println!("event: {:?}", event),
            }
        }

        Ok("")
    }
}

fn main() {
    let mut cane = Changelog::new();
    let parsed = cane.parse("asd");
    match parsed {
        Ok(p) => println!("ciao {}", p),
        Err(e) => println!("error: {}", e),
    }
    // println!("{:?}", cane);

    let j = serde_json::to_string(&cane).unwrap();
    println!("{}", j);
}
