mod widgets;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use futures::{FutureExt, StreamExt};
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode::Char;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Backend;
use ratatui::style::Style;
use ratatui::widgets::{Block, List, ListDirection, ListState, Scrollbar, ScrollbarState};
use ratatui::{prelude::CrosstermBackend, widgets::Paragraph};
use std::sync::Arc;
use strum::EnumIter;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}
pub struct Tui {
    pub terminal: ratatui::DefaultTerminal,
    pub task: Option<JoinHandle<()>>,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
}

impl Tui {
    pub fn start(&mut self) {
        // let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        // let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        let _event_tx = self.event_tx.clone();
        self.task = Some(tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            // let mut tick_interval = tokio::time::interval(tick_delay);
            // let mut render_interval = tokio::time::interval(render_delay);
            _event_tx.send(Event::Init);
            loop {
                // let tick_delay = tick_interval.tick();
                // let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) =>
                        match evt {
                          crossterm::event::Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                              _event_tx.send(Event::Key(key)).unwrap();
                            }
                          },
                          _ => { },
                        }

                      Some(Err(_)) => {
                        _event_tx.send(Event::Error).unwrap();
                      }
                      None => {},
                    }
                  },
                  // _ = tick_delay => {
                  //     _event_tx.send(Event::Tick).unwrap();
                  // },
                  // _ = render_delay => {
                  //     _event_tx.send(Event::Render).unwrap();
                  // },
                }
            }
        }));
    }
    // pub fn tick_rate(self, v: f64) -> Self {
    //     Self {
    //         terminal: self.terminal,
    //         task: self.task,
    //         event_rx: self.event_rx,
    //         event_tx: self.event_tx,
    //         frame_rate: self.frame_rate,
    //         tick_rate: v,
    //     }
    // }
    // pub fn frame_rate(self, v: f64) -> Self {
    //     Self {
    //         terminal: self.terminal,
    //         task: self.task,
    //         event_rx: self.event_rx,
    //         event_tx: self.event_tx,
    //         frame_rate: v,
    //         tick_rate: self.tick_rate,
    //     }
    // }
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        Self {
            event_rx: rx,
            event_tx: tx,
            terminal: ratatui::init(),
            frame_rate: 30.0,
            tick_rate: 1.0,
            task: None,
        }
    }
    pub async fn next(&mut self) -> Option<Event> {
        return self.event_rx.recv().await;
    }
    pub fn exit(self) {
        if let Some(task) = self.task {
            task.abort();
        }
    }
}

fn handle_event(app: &mut App, event: Event) {
    // switch tabline -> SHIFT + H/L
    // switch between selectable widgets -> CTRL + H/J/K/L
    match event {
        Event::Key(key) => match (key.code, key.modifiers) {
            (Char('q'), KeyModifiers::NONE) | (KeyCode::Esc, KeyModifiers::NONE) => {
                app.should_quit = true
            }
            (Char('h'), KeyModifiers::SHIFT) => app.selected_tab.next(),
            // crossterm::event::KeyCode::Esc => app.should_quit = true,
            // Char('q') => app.should_quit = true,
            _ => {}
        },
        _ => {}
    };
}
const TABLINE_LAYOUT: [Tabline; 2] = [Tabline::Chatting, Tabline::FriendRequests];
enum Tabline {
    Chatting,
    FriendRequests,
}
// impl Default for Tabline {
//     fn default() -> Self {
// Self::Chatting(ContactPage::default())
//     }
// }
impl Tabline {
    fn left(&mut self) {}
    fn right(&mut self) {}
    fn up(&mut self) {}
    fn down(&mut self) {}
}
#[derive(Default)]
enum ContactPage {
    #[default]
    ContactList,
    Chat,
    CallButton,
}
#[derive(Default)]
enum FriendRequestPage {
    #[default]
    RequestList,
    Search,
}
fn ui(f: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
        .split(f.area());
    // Tabline
    let tabline = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[0].offset(ratatui::layout::Offset { x: 0, y: 1 }));
    f.render_widget(Paragraph::new("Chatting").centered(), tabline[0]);
    f.render_widget(Paragraph::new("Friend requests").centered(), tabline[1]);

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Fill(1)])
        .split(layout[1]);
    // contacts
    let contact_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
        .split(main_layout[0]);

    let contact_list = List::new(app.contacts.clone())
        .block(Block::bordered().title("Contacts"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);
    f.render_stateful_widget(contact_list, contact_layout[1], &mut app.selected_contact);

    let vertical_scroll = app.selected_contact.selected().unwrap_or(0); // from app state
    let mut scrollbar_state = ScrollbarState::new(app.contacts.len()).position(vertical_scroll);
    let contact_scroll_bar =
        Scrollbar::default().orientation(ratatui::widgets::ScrollbarOrientation::VerticalLeft);
    f.render_stateful_widget(contact_scroll_bar, contact_layout[0], &mut scrollbar_state);
    // friend list
}
// App state
struct App {
    selected_tab: Tabline,
    contact_page: ContactPage,
    friend_request_page: FriendRequestPage,
    selected_contact: ListState,
    contacts: Vec<String>,
    should_quit: bool,
}
pub async fn run() -> anyhow::Result<()> {
    // ratatui terminal
    let mut tui = Tui::new();
    tui.start();

    // application state
    let mut app = App {
        selected_tab: Tabline::default(),
        friend_request_page: FriendRequestPage::default(),
        contact_page: ContactPage::default(),
        should_quit: false,
        contacts: vec!["Mark".to_string(), "Zuckerlizard".to_string()],
        selected_contact: ListState::default().with_selected(Some(0)),
    };

    loop {
        let event = tui.next().await; // blocks until next event
        let Some(event) = event else {
            continue;
        };
        // application update
        handle_event(&mut app, event);

        tui.terminal.draw(|f| {
            ui(f, &mut app);
        })?;

        // application exit
        if app.should_quit {
            break;
        }
    }
    tui.exit();

    Ok(())
}
