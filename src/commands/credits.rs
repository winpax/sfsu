use std::{fmt::Display, io::stdout};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, Padding, Paragraph},
    Frame, Terminal,
};
use sprinkles::inline_const;

mod contributors {
    include!(concat!(env!("OUT_DIR"), "/contributors.rs"));
}

mod packages {
    include!(concat!(env!("OUT_DIR"), "/packages.rs"));
}

struct Url<T: Display> {
    text: T,
    url: String,
}

impl<T: Display> Url<T> {
    fn new(text: T, url: String) -> Self {
        Self { text, url }
    }
}

impl<T: Display> Display for Url<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            self.url, self.text
        )
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Show packages")]
    packages: bool,
}

impl super::Command for Args {
    async fn runner(self) -> anyhow::Result<()> {
        self.terminal_ui()?;

        Ok(())
    }
}

impl Args {
    fn terminal_ui(&self) -> anyhow::Result<()> {
        const TITLE_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let title = format!(
            "🚀 sfsu v{}, created by Juliette Cordor 🚀",
            env!("CARGO_PKG_VERSION")
        );

        let mut items = vec![
            Text::styled(
                "Press Q to exit",
                inline_const![
                    Style
                    Style::new().add_modifier(Modifier::ITALIC)
                ],
            ),
            Text::raw(""),
            Text::styled(
                "💖 Many thanks to everyone who has contributed 💖",
                TITLE_STYLE,
            ),
        ];

        items.extend(
            contributors::CONTRIBUTORS
                .into_iter()
                .map(|(name, url)| Text::from(format!("{name} ({url})"))),
        );

        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|f| self.ui(f, &title, &items))?;
            should_quit = self.handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_events(&self) -> anyhow::Result<bool> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn ui(&self, frame: &mut Frame<'_>, title: &str, items: &[Text<'_>]) {
        // println!();

        // if self.packages {
        //     println!("📦📦📦 sfsu is built with the following packages 📦📦📦");
        //     for (name, version) in packages::PACKAGES {
        //         let url = Url::new(name, format!("https://crates.io/crates/{name}"));
        //         println!("{url}: {version}");
        //     }

        //     println!();
        // }

        // println!("💖💖💖 Many thanks to everyone who as contributed to sfsu 💖💖💖");
        // for (name, url) in contributors::CONTRIBUTORS {
        //     let url = Url::new(name, url.to_string());

        //     println!("{url}");
        // }

        frame.render_widget(
            List::new(
                items
                    .iter()
                    .cloned()
                    .map(|text| text.alignment(Alignment::Center)),
            )
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            ),
            frame.size(),
        );
    }
}
