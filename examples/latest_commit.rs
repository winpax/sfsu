use chrono::{DateTime, FixedOffset};
use git2::Repository;

fn parse_time(secs: i64, offset: i32) -> Option<DateTime<FixedOffset>> {
    let naive_time = DateTime::from_timestamp(secs, 0)?;

    let offset = FixedOffset::east_opt(offset * 60)?;

    Some(naive_time.with_timezone(&offset))
}

fn main() -> Result<(), git2::Error> {
    // Open the local repository
    let repo = Repository::open(".")?;

    // Get the current HEAD
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    // Get the commit time
    let time = commit.time();
    let time = parse_time(time.seconds(), time.offset_minutes()).unwrap();
    println!("Commit time: {}", time);

    // Get the commit author
    let author = commit.author();
    println!("Author name: {}", author.name().unwrap_or("No name"));
    println!("Author email: {}", author.email().unwrap_or("No email"));

    Ok(())
}
