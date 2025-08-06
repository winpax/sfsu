use clap::{Parser, ValueEnum};
use rayon::prelude::*;

use sprinkles::contexts::ScoopContext;

use crate::{models::min::Info, output::structured::Structured};

#[derive(Debug, Clone, Parser)]
/// List all installed packages
pub struct Args {
    #[cfg(not(feature = "v2"))]
    #[clap(
        help = format!("The pattern to search for (can be a regex). {}", console::style("DEPRECATED: Use sfsu search --installed. Will be removed in v2").yellow())
    )]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively list packages in")]
    bucket: Option<String>,

    #[clap(long, help = "Sort by the given field", default_value = "name")]
    sort_by: SortBy,

    #[clap(long, help = "Sort in descending order")]
    descending: bool,

    #[clap(from_global)]
    json: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum SortBy {
    Name,
    Version,
    Source,
    Updated,
    Notes,
}

impl SortBy {
    pub fn sort(self, a: &Info, b: &Info, descending: bool) -> std::cmp::Ordering {
        let ordering = match self {
            SortBy::Name => Self::sort_name(a, b),
            SortBy::Version => Self::sort_version(a, b),
            SortBy::Source => Self::sort_source(a, b),
            SortBy::Updated => Self::sort_updated(a, b),
            SortBy::Notes => Self::sort_notes(a, b),
        };

        if descending {
            ordering
        } else {
            ordering.reverse()
        }
    }

    fn sort_name(a: &Info, b: &Info) -> std::cmp::Ordering {
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    }

    fn sort_version(a: &Info, b: &Info) -> std::cmp::Ordering {
        // TODO: Proper semantic version sorting
        a.version.cmp(&b.version)
    }

    fn sort_source(a: &Info, b: &Info) -> std::cmp::Ordering {
        // TODO: Unknown source should sort first
        a.source.cmp(&b.source)
    }

    fn sort_updated(a: &Info, b: &Info) -> std::cmp::Ordering {
        a.updated.cmp(&b.updated)
    }

    fn sort_notes(a: &Info, b: &Info) -> std::cmp::Ordering {
        a.notes.cmp(&b.notes)
    }
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let mut outputs = Info::list_installed(ctx, self.bucket.as_ref())?;

        outputs.par_sort_unstable_by(|a, b| self.sort_by.sort(a, b, self.descending));

        if self.json {
            let output_json = serde_json::to_string_pretty(&outputs)?;

            println!("{output_json}");
        } else {
            if outputs.is_empty() {
                println!("No packages found.");
                return Ok(());
            }

            let values = outputs
                .into_par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            let outputs = Structured::new(&values);

            print!("{outputs}");
        }

        Ok(())
    }
}
