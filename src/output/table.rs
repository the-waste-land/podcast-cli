use prettytable::{Cell, Row, Table};

use crate::api::types::{Category, Episode, Podcast, Stats};
use crate::commands::youtube_meta::YoutubeMetaItem;

const CLIP: usize = 72;

pub fn render_podcast_list(podcasts: &[Podcast]) -> String {
    if podcasts.is_empty() {
        return "No podcasts found.".to_string();
    }

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("Title"),
        Cell::new("Author"),
        Cell::new("Language"),
    ]));

    for podcast in podcasts {
        table.add_row(Row::new(vec![
            Cell::new(&podcast.id.to_string()),
            Cell::new(&clip(&podcast.title)),
            Cell::new(&clip(&podcast.author)),
            Cell::new(&value_or_dash(&podcast.language)),
        ]));
    }

    table.to_string()
}

pub fn render_podcast_detail(podcast: &Podcast) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("Field"), Cell::new("Value")]));
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new(&podcast.id.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Title"),
        Cell::new(&value_or_dash(&podcast.title)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Author"),
        Cell::new(&value_or_dash(&podcast.author)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Language"),
        Cell::new(&value_or_dash(&podcast.language)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feed URL"),
        Cell::new(&value_or_dash(&podcast.feed_url)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Website"),
        Cell::new(&value_or_dash(&podcast.website)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Description"),
        Cell::new(&clip(&podcast.description)),
    ]));

    table.to_string()
}

pub fn render_episode_list(episodes: &[Episode]) -> String {
    if episodes.is_empty() {
        return "No episodes found.".to_string();
    }

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("Title"),
        Cell::new("Feed"),
        Cell::new("Published"),
        Cell::new("Duration"),
    ]));

    for episode in episodes {
        table.add_row(Row::new(vec![
            Cell::new(&option_u64(episode.id)),
            Cell::new(&clip_optional(episode.title.as_deref())),
            Cell::new(&clip_optional(episode.feed_title.as_deref())),
            Cell::new(&published_value(episode)),
            Cell::new(&duration_value(episode.duration)),
        ]));
    }

    table.to_string()
}

pub fn render_episode_detail(episode: &Episode) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("Field"), Cell::new("Value")]));
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new(&option_u64(episode.id)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Title"),
        Cell::new(&value_or_dash_opt(episode.title.as_deref())),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feed ID"),
        Cell::new(&option_u64(episode.feed_id)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feed Title"),
        Cell::new(&value_or_dash_opt(episode.feed_title.as_deref())),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Published"),
        Cell::new(&published_value(episode)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Duration"),
        Cell::new(&duration_value(episode.duration)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Link"),
        Cell::new(&value_or_dash_opt(episode.link.as_deref())),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Description"),
        Cell::new(&clip_optional(episode.description.as_deref())),
    ]));

    table.to_string()
}

pub fn render_categories_list(categories: &[Category]) -> String {
    if categories.is_empty() {
        return "No categories found.".to_string();
    }

    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("ID"), Cell::new("Name")]));

    for category in categories {
        table.add_row(Row::new(vec![
            Cell::new(&option_u64(category.id)),
            Cell::new(&value_or_dash_opt(category.name.as_deref())),
        ]));
    }

    table.to_string()
}

pub fn render_stats(stats: &Stats) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("Metric"), Cell::new("Value")]));
    table.add_row(Row::new(vec![
        Cell::new("Total Feeds"),
        Cell::new(&format_optional_count(stats.feed_count_total)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Total Episodes"),
        Cell::new(&format_optional_count(stats.episode_count_total)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feeds Updated (3d)"),
        Cell::new(&format_optional_count(stats.feeds_with_new_episodes_3days)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feeds Updated (10d)"),
        Cell::new(&format_optional_count(stats.feeds_with_new_episodes_10days)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feeds Updated (30d)"),
        Cell::new(&format_optional_count(stats.feeds_with_new_episodes_30days)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Feeds With Value Blocks"),
        Cell::new(&format_optional_count(stats.feeds_with_value_blocks)),
    ]));

    table.to_string()
}

pub fn render_youtube_meta(item: &YoutubeMetaItem) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("Field"), Cell::new("Value")]));
    table.add_row(Row::new(vec![
        Cell::new("Video ID"),
        Cell::new(&item.video_id),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Title"),
        Cell::new(&value_or_dash_opt(item.title.as_deref())),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Channel"),
        Cell::new(&value_or_dash_opt(item.channel.as_deref())),
    ]));
    table.add_row(Row::new(vec![Cell::new("URL"), Cell::new(&item.url)]));
    table.add_row(Row::new(vec![
        Cell::new("Duration"),
        Cell::new(
            &item
                .duration
                .map(|seconds| format!("{seconds}s"))
                .unwrap_or_else(|| "-".to_string()),
        ),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Upload Date"),
        Cell::new(&value_or_dash_opt(item.upload_date.as_deref())),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Timestamp"),
        Cell::new(&option_i64(item.timestamp)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Views"),
        Cell::new(&option_u64(item.view_count)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Likes"),
        Cell::new(&option_u64(item.like_count)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Comments"),
        Cell::new(&option_u64(item.comment_count)),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Availability"),
        Cell::new(&value_or_dash_opt(item.availability.as_deref())),
    ]));

    table.to_string()
}

fn clip(value: &str) -> String {
    if value.chars().count() <= CLIP {
        return value_or_dash(value);
    }

    let clipped = value.chars().take(CLIP - 3).collect::<String>();
    format!("{clipped}...")
}

fn value_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn value_or_dash_opt(value: Option<&str>) -> String {
    value.map(value_or_dash).unwrap_or_else(|| "-".to_string())
}

fn clip_optional(value: Option<&str>) -> String {
    value.map(clip).unwrap_or_else(|| "-".to_string())
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|raw| raw.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn option_i64(value: Option<i64>) -> String {
    value
        .map(|raw| raw.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn duration_value(duration_seconds: Option<u32>) -> String {
    duration_seconds
        .map(|seconds| format!("{seconds}s"))
        .unwrap_or_else(|| "-".to_string())
}

fn published_value(episode: &Episode) -> String {
    if let Some(pretty) = episode.date_published_pretty.as_deref() {
        return value_or_dash(pretty);
    }

    episode
        .date_published
        .map(|timestamp| timestamp.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_count(value: Option<u64>) -> String {
    value.map(format_count).unwrap_or_else(|| "-".to_string())
}

fn format_count(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }

    grouped.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn format_count_uses_thousands_separator() {
        assert_eq!(super::format_count(1_234_567_890), "1,234,567,890");
    }

    #[test]
    fn format_count_handles_small_numbers() {
        assert_eq!(super::format_count(42), "42");
        assert_eq!(super::format_count(0), "0");
    }
}
