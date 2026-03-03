use prettytable::{Cell, Row, Table};

use crate::api::types::Podcast;

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
