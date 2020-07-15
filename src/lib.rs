// wikidumpgrep
//
// (C) 2020 Count Count
//
// Distributed under the terms of the MIT license.

use quick_xml::events::Event;
use quick_xml::Reader;
use regex::RegexBuilder;
use std::fs::File;
use std::{io::BufRead, io::BufReader, str::from_utf8_unchecked};

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn from_unicode(s: &[u8]) -> &str {
    unsafe { from_utf8_unchecked(s) }
}

fn read_text_and_then<T: BufRead, ResT, F>(reader: &mut Reader<T>, buf: &mut Vec<u8>, mut f: F) -> ResT
where
    F: FnMut(&str) -> ResT,
{
    if let Event::Text(escaped_text) = reader.read_event(buf).unwrap() {
        let unescaped_text = escaped_text.unescaped().unwrap();
        let text = from_unicode(&unescaped_text);
        f(text)
    } else {
        panic!("Text expected");
    }
}

pub fn search_dump(regex: &str, dump_file: &str, namespaces: &[&str]) {
    let re = RegexBuilder::new(regex).build().unwrap();
    let buf_size = 2 * 1024 * 1024;
    let file = File::open(&dump_file).unwrap();
    let buf_reader = BufReader::with_capacity(buf_size, file);
    let mut reader = Reader::from_reader(buf_reader);

    let mut buf: Vec<u8> = Vec::with_capacity(1000 * 1024);
    let mut title: String = String::with_capacity(10000);
    loop {
        match reader.read_event(&mut buf).unwrap() {
            Event::Start(ref e) => match e.name() {
                b"title" => {
                    read_text_and_then(&mut reader, &mut buf, |text| {
                        title.clear();
                        title.push_str(text);
                    });
                }
                b"ns" => {
                    let skip = read_text_and_then(&mut reader, &mut buf, |text| {
                        !namespaces.is_empty() && !namespaces.iter().any(|&i| i == text)
                    });
                    if skip {
                        reader.read_to_end(b"page", &mut buf).unwrap();
                    }
                }
                b"text" => {
                    read_text_and_then(&mut reader, &mut buf, |text| {
                        if re.is_match(text) {
                            println!("* [[{}]]", title.as_str());
                        }
                    });
                }
                _other_tag => { /* ignore */ }
            },
            Event::Eof => {
                break;
            }
            _other_event => (),
        }
        buf.clear();
    }
}
