# Reddit Monitor

An app that checks every n seconds for the newest posts in a subreddit's front page and fetches them.

## Usage with Rust

```rust
cargo run <subreddit_name> [sort_order] [interval_in_seconds]
```

## Usage with Docker

```console
docker pull petrughionea/projects:reddit-monitor

# example command
docker run --rm reddit-monitor programming hot 10
```
