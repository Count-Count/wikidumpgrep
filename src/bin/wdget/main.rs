// wdget
//
// (C) 2020 Count Count
//
// Distributed under the terms of the MIT license.

mod lib;

use std::env::current_dir;
use std::process;

use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use lazy_static::lazy_static;
use lib::*;
use regex::Regex;
use reqwest::Client;
use termcolor::ColorChoice;

fn create_client() -> Result<Client> {
    Ok(reqwest::Client::builder()
        .user_agent(concat!(
            "wdget/",
            crate_version!(),
            " (https://github.com/Count-Count/wikidumptools)"
        ))
        .build()?)
}

async fn list_wikis(client: &Client) -> Result<()> {
    let mut wikis = get_available_wikis_from_wikidata(client).await?;
    wikis.sort_unstable_by(|e1, e2| e1.id.cmp(&e2.id));
    for ref wiki in wikis {
        println!("{} - {}", wiki.id.as_str(), wiki.name.as_str());
    }
    Ok(())
}

async fn list_dates(client: &Client, wiki: &str) -> Result<()> {
    let dates = get_available_dates(client, wiki).await?;
    for date in dates {
        println!("{}", date);
    }
    Ok(())
}

async fn list_types(client: &Client, wiki: &str, date: &str) -> Result<()> {
    let dump_status = get_dump_status(client, wiki, date).await?;
    for (job_name, job_info) in &dump_status.jobs {
        if let Some(files) = &job_info.files {
            let sum = files.values().map(|info| info.size.unwrap_or(0)).sum::<u64>();
            println!(
                "{} - status: {} - size: {:.2} MiB",
                &job_name,
                &job_info.status,
                sum as f64 / 1024.0 / 1024.0
            );
        } else {
            println!("{} - status: {}", &job_name, &job_info.status);
        }
    }
    Ok(())
}

async fn check_date_may_retrieve_latest(
    client: &Client,
    wiki: &str,
    date_spec: &str,
    dump_type: Option<&str>,
) -> Result<String> {
    if date_spec == "latest" {
        return Ok(get_latest_available_date(client, wiki, dump_type).await?);
    } else {
        lazy_static! {
            static ref RE: Regex = Regex::new("[1-9][0-9]{7}$").expect("Error parsing dump date regex");
        }
        if RE.is_match(date_spec) {
            Ok(date_spec.to_owned())
        } else {
            Err(anyhow::Error::from(Error::InvalidDumpDate()))
        }
    }
}

async fn run() -> Result<()> {
    let wiki_name_arg = Arg::new("wiki name").about("Name of the wiki").required(true);
    let dump_date_arg = Arg::new("dump date")
        .about("Date of the dump (YYYYMMDD or 'latest')")
        .required(true);

    let matches = App::new("WikiDumpGet")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Download Wikipedia and other Wikimedia wiki dumps from the internet.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .about("Don't print progress updates."),
        )
        .subcommand(
            App::new("download")
                .about("Download a wiki dump")
                .arg(wiki_name_arg.clone())
                .arg(dump_date_arg.clone())
                .arg(Arg::new("dump type").about("Type of the dump").required(true))
                .arg(
                    Arg::new("mirror")
                        .long("mirror")
                        .about("Root mirror URL")
                        .takes_value(true)
                        .max_values(1),
                )
                .arg(
                    Arg::new("decompress")
                        .short('d')
                        .long("decompress")
                        .about("Decompress .bz2 files during download"),
                ),
        )
        .subcommand(App::new("list-wikis").about("List all wikis for which dumps are available"))
        .subcommand(
            App::new("list-dates")
                .about("List all dump dates available for this wiki")
                .arg(wiki_name_arg.clone())
                .arg(Arg::new("dump type").about("Type of the dump").required(false)),
        )
        .subcommand(
            App::new("list-dumps")
                .about("List all dumps available for this wiki at this date")
                .arg(wiki_name_arg.clone())
                .arg(dump_date_arg),
        )
        .subcommand(App::new("list-mirrors").about("List available mirrors"))
        .get_matches();

    let _color_choice = if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    let client = create_client()?;
    match matches.subcommand_name().unwrap() {
        "list-wikis" => list_wikis(&client).await?,

        "list-dates" => {
            // todo: check args: wiki name, handle optional type, handle no dump found condition
            let subcommand_matches = matches.subcommand_matches("list-dates").unwrap();
            list_dates(&client, subcommand_matches.value_of("wiki name").unwrap()).await?;
        }

        "list-dumps" => {
            // todo: check args: wiki name; handle wiki/date not found, dump status file does not exist (yet)
            let subcommand_matches = matches.subcommand_matches("list-dumps").unwrap();
            let wiki = subcommand_matches.value_of("wiki name").unwrap();
            let date_spec = subcommand_matches.value_of("dump date").unwrap();
            let date = check_date_may_retrieve_latest(&client, wiki, date_spec, None).await?;
            eprintln!("Listing dumps for {}, dump run from {}", wiki, date);
            list_types(&client, wiki, &date).await?
        }

        "download" => {
            // todo: check args
            let subcommand_matches = matches.subcommand_matches("download").unwrap();
            let wiki = subcommand_matches.value_of("wiki name").unwrap();
            let date_spec = subcommand_matches.value_of("dump date").unwrap();
            let dump_type = subcommand_matches.value_of("dump type").unwrap();
            let date = check_date_may_retrieve_latest(&client, wiki, date_spec, Some(dump_type)).await?;
            let current_dir = current_dir().map_err(|e| anyhow!("Current directory not accessible: {}", e))?;
            let download_options = DownloadOptions {
                mirror: subcommand_matches.value_of("mirror"),
                verbose: !matches.is_present("quiet"),
                keep_partial: false,
                resume_partial: false,
                decompress: subcommand_matches.is_present("decompress"),
            };
            download(&client, wiki, &date, dump_type, current_dir, &download_options).await?
        }
        _ => unreachable!("Unknown subcommand, should be caught by arg matching."),
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let res = run().await;
    if let Err(e) = res {
        eprintln!("{}", e);
        process::exit(1);
    }
}
