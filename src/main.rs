extern crate clap;

use ::http as thehttp;
use anyhow::{Error, Result};
use clap::{App, Arg, ArgMatches};
use hyper::header::HeaderMap;
use regex::Regex;
use std::str::FromStr;
use tokio::time::Duration;

mod bench;
mod error;
mod http;
mod proto;
mod results;
mod runtime;
mod utils;

use crate::http::BenchType;

/// Matches a string like '12d 24h 5m 45s' to a regex capture.
static DURATION_MATCH: &str =
    "(?P<days>[0-9]+)d|(?P<hours>[0-9]+)h|(?P<minutes>[0-9]+)m|(?P<seconds>[0-9]+)s";

/// ReWrk
///
/// Captures CLI arguments and build benchmarking settings and runtime to
/// suite the arguments and options.
fn main() {
    let args = parse_args();

    let threads: usize = match args.value_of("threads").unwrap_or("1").parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("invalid parameter for 'threads' given, input type must be a integer.");
            return;
        }
    };

    let conns: usize = match args.value_of("connections").unwrap_or("1").parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("invalid parameter for 'connections' given, input type must be a integer.");
            return;
        }
    };

    let host: &str = match args.value_of("host") {
        Some(v) => v,
        None => {
            eprintln!("missing 'host' parameter.");
            return;
        }
    };

    let mut h = HeaderMap::new();
    let headers: HeaderMap = match args.values_of("header") {
        Some(v) => {
            for s in v {
                let ss: Vec<&str> = s.splitn(2, ":").collect();
                if ss.len() != 2 {
                    continue;
                }
                eprintln!("header applied: {}: {}", &ss[0], &ss[1].trim());
                h.insert(
                    thehttp::header::HeaderName::from_str(&ss[0]).unwrap(),
                    thehttp::header::HeaderValue::from_str(&ss[1].trim()).unwrap(),
                );
            }
            h
        }
        None => h,
    };

    let http2: bool = args.is_present("http2");
    let json: bool = args.is_present("json");

    let bench_type = if http2 {
        BenchType::HTTP2
    } else {
        BenchType::HTTP1
    };

    let duration: &str = args.value_of("duration").unwrap_or("1s");
    let duration = match parse_duration(duration) {
        Ok(dur) => dur,
        Err(e) => {
            eprintln!("failed to parse duration parameter: {}", e);
            return;
        }
    };

    let pct: bool = args.is_present("pct");

    let rounds: usize = args
        .value_of("rounds")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap_or(1);

    let settings = bench::BenchmarkSettings {
        threads,
        connections: conns,
        host: host.to_string(),
        bench_type,
        duration,
        display_percentile: pct,
        display_json: json,
        rounds,
        headers,
    };

    bench::start_benchmark(settings);
}

/// Parses a duration string from the CLI to a Duration.
/// '11d 3h 32m 4s' -> Duration
///
/// If no matches are found for the string or a invalid match
/// is captured a error message returned and displayed.
fn parse_duration(duration: &str) -> Result<Duration> {
    let mut dur = Duration::default();

    let re = Regex::new(DURATION_MATCH).unwrap();
    for cap in re.captures_iter(duration) {
        let add_to = if let Some(days) = cap.name("days") {
            let days = days.as_str().parse::<u64>()?;

            let seconds = days * 24 * 60 * 60;
            Duration::from_secs(seconds)
        } else if let Some(hours) = cap.name("hours") {
            let hours = hours.as_str().parse::<u64>()?;

            let seconds = hours * 60 * 60;
            Duration::from_secs(seconds)
        } else if let Some(minutes) = cap.name("minutes") {
            let minutes = minutes.as_str().parse::<u64>()?;

            let seconds = minutes * 60;
            Duration::from_secs(seconds)
        } else if let Some(seconds) = cap.name("seconds") {
            let seconds = seconds.as_str().parse::<u64>()?;

            Duration::from_secs(seconds)
        } else {
            return Err(Error::msg(format!("invalid match: {:?}", cap)));
        };

        dur += add_to
    }

    if dur.as_secs() == 0 {
        return Err(Error::msg(format!(
            "failed to extract any valid duration from {}",
            duration
        )));
    }

    Ok(dur)
}

/// Contains Clap's app setup.
fn parse_args() -> ArgMatches {
    App::new("ReWrk")
        .version("0.3.1")
        .author("Harrison Burt <hburt2003@gmail.com>")
        .about("Benchmark HTTP/1 and HTTP/2 frameworks without pipelining bias.")
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .about("Set the amount of threads to use e.g. '-t 12'")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::new("connections")
                .short('c')
                .long("connections")
                .about("Set the amount of concurrent e.g. '-c 512'")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .about("Set the host to bench e.g. '-h http://127.0.0.1:5050'")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("header")
                .short('H')
                .long("header")
                .about("Add header to request e.g. 'User-Agent: wrk'")
                .takes_value(true)
                .multiple_occurrences(true)
                .required(false)
                .min_values(0),
        )
        .arg(
            Arg::new("http2")
                .long("http2")
                .about("Set the client to use http2 only. (default is http/1) e.g. '--http2'")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                .about("Set the duration of the benchmark.")
                .takes_value(true)
                .default_value("10s")
                .required(true),
        )
        .arg(
            Arg::new("pct")
                .long("pct")
                .about("Displays the percentile table after benchmarking.")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .about("Displays the results in a json format")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::new("rounds")
                .long("rounds")
                .about("Repeats the benchmarks n amount of times")
                .takes_value(true)
                .required(false),
        )
        //.arg(
        //    Arg::new("random")
        //        .long("rand")
        //        .about(
        //            "Sets the benchmark type to random mode, \
        //             clients will randomly connect and re-connect.\n\
        //             NOTE: This will cause the HTTP2 flag to be ignored."
        //        )
        //        .takes_value(false)
        //        .required(false)
        //)
        .get_matches()
}
