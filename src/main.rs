use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::env;
use std::time::Instant;
#[derive(Debug, serde::Deserialize)]
struct RedditFeed {
    data: PostData,
}
#[derive(Debug, serde::Deserialize)]
struct PostData {
    children: Vec<Post>,
}
#[derive(Debug, serde::Deserialize)]
struct Post {
    data: PostAttributes,
}
#[derive(Debug, serde::Deserialize)]
struct PostAttributes {
    title: String,
    created_utc: f64,
    permalink: String,
}
fn timestamp_to_datetime(timestamp: f64) -> DateTime<Utc> {
    Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap())
}
fn download_news_feed(subreddit: &str, sort_by: &str) -> Result<RedditFeed> {
    let reddit_url = format!("https://www.reddit.com/r/{}/{}/.json", subreddit, sort_by);
    let response = ureq::get(&reddit_url).call()?;
    let body = response
        .into_string()
        .context("Failed to read response body")?;
    let json_str =
        serde_json::from_str(&body).with_context(|| format!("Error parsing JSON: {}", body))?;
    Ok(json_str)
}
fn monitor_and_print_news_feed(subreddit: &str, sort_by: &str, interval: u64) {
    let mut printed_posts = HashMap::<String, (DateTime<Utc>, String)>::new();
    let mut last_print_time = Instant::now();
    loop {
        if last_print_time.elapsed().as_secs() >= interval {
            match download_news_feed(subreddit, sort_by) {
                Ok(reddit_feed) => {
                    let posts = &reddit_feed
                        .data
                        .children
                        .iter()
                        .map(|post| &post.data)
                        .collect::<Vec<_>>();
                    let mut new_posts_detected = false;
                    for post in posts {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            printed_posts.entry(post.title.clone())
                        {
                            let creation_date = timestamp_to_datetime(post.created_utc);
                            let title = &post.title;
                            let permalink = format!("https://www.reddit.com{}", &post.permalink);
                            println!("Title: {}", title);
                            println!("Creation Date (UTC): {}", creation_date);
                            println!("Link: {}\n", permalink);
                            e.insert((creation_date, permalink));
                            new_posts_detected = true;
                        }
                    }
                    if !new_posts_detected {
                        println!("No new posts detected.");
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
            last_print_time = Instant::now();
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        eprintln!("Error: Incorrect number of arguments(please use the 'help' command)");
        std::process::exit(1);
    }
    let command = &args[1];
    match command.as_str() {
        "help" => {
            println!("Usage: cargo run <subreddit_name> [sort_order] [interval_in_seconds]");
            println!("Usage: cargo run help (to display this help message)");
            std::process::exit(0);
        }
        _ => {
            let subreddit = command;
            let sort_by = args.get(2).map_or("hot", |s| s.as_str());
            let interval: u64 = args.get(3).map_or(15, |s| {
                s.parse().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid value for interval");
                    std::process::exit(1);
                })
            });
            monitor_and_print_news_feed(subreddit, sort_by, interval);
        }
    }
}
