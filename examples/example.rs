mod util;

use crate::util::StatefulTree;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};
use std::{error::Error, io, time::{Duration, Instant}};

use ratatui_tree_widget::{Tree, TreeItem};

struct App<'a> {
    tree: StatefulTree<'a>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            tree: StatefulTree::with_items(vec![
                TreeItem::new_leaf("a"),
                TreeItem::new(
                    "b",
                    vec![
                        TreeItem::new_leaf("c"),
                        TreeItem::new("d", vec![TreeItem::new_leaf("e"), TreeItem::new_leaf("f")]),
                        TreeItem::new_leaf("g"),
                    ],
                ),
                TreeItem::new_leaf("h"),
            ]),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_key_time = Instant::now(); // Track the last key press time

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let items = Tree::new(app.tree.items.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Tree Widget {:?}", app.tree.state)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(items, area, &mut app.tree.state);
        })?;

        if event::poll(Duration::from_millis(149)).ok().unwrap() {
            if let Event::Key(key) = event::read().ok().unwrap() {
                // Debounce threshold
                if last_key_time.elapsed() >= Duration::from_millis(150) {
                    // Update the last key press time
                    last_key_time = Instant::now(); 
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('\n' | ' ') => app.tree.toggle(),
                        KeyCode::Left => app.tree.left(),
                        KeyCode::Right => app.tree.right(),
                        KeyCode::Down => app.tree.down(),
                        KeyCode::Up => app.tree.up(),
                        KeyCode::Home => app.tree.first(),
                        KeyCode::End => app.tree.last(),
                        _ => {}
                    }
                }
            }
        }
    }
}
