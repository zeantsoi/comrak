//! The `comrak` binary.

// #![deny(missing_docs,
//         missing_debug_implementations,
// 	missing_copy_implementations,
// 	trivial_casts,
// 	trivial_numeric_casts,
// 	unsafe_code,
// 	unstable_features,
// 	unused_import_braces,
// 	unused_qualifications)]

#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![allow(unknown_lints, doc_markdown, cyclomatic_complexity)]

// When compiled for the rustc compiler itself we want to make sure that this is
// an unstable crate.
#![cfg_attr(rustbuild, feature(staged_api, rustc_private))]
#![cfg_attr(rustbuild, unstable(feature = "rustc_private", issue = "27812"))]

extern crate entities;
#[macro_use]
extern crate clap;
extern crate unicode_categories;
extern crate typed_arena;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod arena_tree;
mod html;
mod cm;
mod parser;
mod nodes;
mod ctype;
mod scanners;
mod strings;
mod entity;

use std::collections::BTreeSet;
use std::io::Read;
use std::process;
use typed_arena::Arena;

fn main() {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            clap::Arg::with_name("file")
                .value_name("FILE")
                .multiple(true)
                .help(
                    "The CommonMark file to parse; or standard input if none passed",
                ),
        )
        .arg(clap::Arg::with_name("hardbreaks").long("hardbreaks").help(
            "Treat newlines as hard line breaks",
        ))
        .arg(
            clap::Arg::with_name("github-pre-lang")
                .long("github-pre-lang")
                .help("Use GitHub-style <pre lang> for code blocks"),
        )
        .arg(
            clap::Arg::with_name("extension")
                .short("e")
                .long("extension")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .possible_values(
                    &[
                        "strikethrough",
                        "tagfilter",
                        "table",
                        "autolink",
                        "tasklist",
                        "superscript",
                    ],
                )
                .value_name("EXTENSION")
                .help("Specify an extension name to use"),
        )
        .arg(
            clap::Arg::with_name("format")
                .short("t")
                .long("to")
                .takes_value(true)
                .possible_values(&["html", "commonmark"])
                .default_value("html")
                .value_name("FORMAT")
                .help("Specify output format"),
        )
        .arg(
            clap::Arg::with_name("width")
                .long("width")
                .takes_value(true)
                .value_name("WIDTH")
                .default_value("0")
                .help("Specify wrap width (0 = nowrap)"),
        )
        .get_matches();

    let mut exts = matches.values_of("extension").map_or(
        BTreeSet::new(),
        |vals| vals.collect(),
    );

    let options = parser::ComrakOptions {
        hardbreaks: matches.is_present("hardbreaks"),
        github_pre_lang: matches.is_present("github-pre-lang"),
        width: matches.value_of("width").unwrap_or("0").parse().unwrap_or(
            0,
        ),
        ext_strikethrough: exts.remove("strikethrough"),
        ext_tagfilter: exts.remove("tagfilter"),
        ext_table: exts.remove("table"),
        ext_autolink: exts.remove("autolink"),
        ext_tasklist: exts.remove("tasklist"),
        ext_superscript: exts.remove("superscript"),
    };

    assert!(exts.is_empty());

    let mut s = String::with_capacity(2048);

    match matches.values_of("file") {
        None => {
            std::io::stdin().read_to_string(&mut s).unwrap();
        }
        Some(fs) => {
            for f in fs {
                let mut io = std::fs::File::open(f).unwrap();
                io.read_to_string(&mut s).unwrap();
            }
        }
    };

    let arena = Arena::new();
    let root = parser::parse_document(&arena, &s, &options);

    let formatter = match matches.value_of("format") {
        Some("html") => html::format_document,
        Some("commonmark") => cm::format_document,
        _ => panic!("unknown format"),
    };

    print!("{}", formatter(root, &options));

    process::exit(0);
}
