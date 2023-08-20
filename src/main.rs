use std::{
    fs::{self, File},
    io,
    path::PathBuf,
    str::FromStr,
};

use chrono::{Local, NaiveDate};
use clap::{builder::ValueParser, Parser};
use csv::Writer;
use reqwest::header::{HeaderMap, HeaderValue};

use uber_earnings::{
    read_session_from_config_file, read_session_from_file,
    serde::{ActivityFeedResponse, ActivityRequest, PaginationOption},
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long("session"), value_name = "FILE")]
    pub session_file: Option<PathBuf>,
    #[arg(long, short, value_name = "FILE")]
    pub output: Option<PathBuf>,
    #[arg(value_parser(parse_date))]
    pub start_date: NaiveDate,
    #[arg(value_parser(parse_date))]
    pub end_date: NaiveDate,
}

fn parse_date(s: &str) -> chrono::ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let writer: Box<dyn io::Write> = if let Some(path) = args.output {
        Box::new(File::create(path)?)
    } else {
        Box::new(io::stdout())
    };

    let mut writer = Writer::from_writer(writer);
    writer.write_record([
        "UUID", "Type", "Date", "Time", "Title", "Total", "Url", "Tip", "Duration", "Distance",
        "Pickup", "DropOff", "MapUrl",
    ])?;

    let session = if let Some(path) = args.session_file {
        read_session_from_file(path)
    } else {
        read_session_from_config_file()
    }?;

    let mut headers = HeaderMap::new();
    headers.insert("x-csrf-token", HeaderValue::from_static("x"));
    headers.insert("Cookie", HeaderValue::from_str(&session).unwrap());

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let mut has_more_data = true;
    let mut cursor: Option<String> = None;

    while has_more_data {
        let json = ActivityRequest {
            start_date_iso: args.start_date,
            end_date_iso: args.end_date,
            pagination_option: cursor.take().map(|cursor| PaginationOption { cursor }),
        };

        let res = client
            .post("https://drivers.uber.com/earnings/api/getWebActivityFeed")
            .json(&json)
            .send()?;

        let json: ActivityFeedResponse = res.json()?;

        match json {
            ActivityFeedResponse::Failure { data } => {
                println!("Error: {}", data.message);
                has_more_data = false;
            }
            ActivityFeedResponse::Success { data } => {
                has_more_data = data.pagination.has_more_data;
                cursor = data.pagination.next_cursor;

                if let Some(activities) = data.activities {
                    for activity in activities {
                        writer.write_record([
                            activity.uuid,
                            activity.type_,
                            activity
                                .recognized_at
                                .with_timezone(&Local)
                                .date_naive()
                                .to_string(),
                            activity
                                .recognized_at
                                .with_timezone(&Local)
                                .time()
                                .to_string(),
                            activity.activity_title,
                            activity.formatted_total,
                            activity.routing.webview_url,
                            match activity.breakdown_details {
                                None => "".to_owned(),
                                Some(ref breakdown) => breakdown.formatted_tip.to_owned(),
                            },
                            match activity.trip_meta_data {
                                None => "".to_owned(),
                                Some(ref meta) => meta.formatted_duration.to_owned(),
                            },
                            match activity.trip_meta_data {
                                None => "".to_owned(),
                                Some(ref meta) => meta.formatted_distance.to_owned(),
                            },
                            match activity.trip_meta_data {
                                None => "".to_owned(),
                                Some(ref meta) => meta.pickup_address.to_owned(),
                            },
                            match activity.trip_meta_data {
                                None => "".to_owned(),
                                Some(ref meta) => meta.drop_off_address.to_owned(),
                            },
                            match activity.trip_meta_data {
                                None => "".to_owned(),
                                Some(ref meta) => meta.map_url.to_owned(),
                            },
                        ])?;
                    }
                }
            }
        }
    }

    writer.flush()?;

    Ok(())
}
