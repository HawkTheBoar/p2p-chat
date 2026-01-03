use crossterm::event::KeyEventKind;
use futures::{FutureExt, StreamExt};
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode::Char;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::prelude::Backend;
use ratatui::{prelude::CrosstermBackend, widgets::Paragraph};
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
    match event {
        Event::Key(key) => match key.code {
            crossterm::event::KeyCode::Esc => app.should_quit = true,
            Char('j') => app.counter += 1,
            Char('k') => app.counter -= 1,
            Char('q') => app.should_quit = true,
            _ => {}
        },
        _ => {}
    };
}
fn ui(f: &mut Frame, app: &App) {
    f.render_widget(
        Paragraph::new(format!("Counter: {}", app.counter)),
        f.area(),
    );
}
// App state
struct App {
    counter: i64,
    should_quit: bool,
}
pub async fn run() -> anyhow::Result<()> {
    // ratatui terminal
    let mut tui = Tui::new();
    tui.start();

    // application state
    let mut app = App {
        counter: 0,
        should_quit: false,
    };

    loop {
        let event = tui.next().await; // blocks until next event
        let Some(event) = event else {
            continue;
        };
        // application update
        handle_event(&mut app, event);

        tui.terminal.draw(|f| {
            ui(f, &app);
        })?;

        // application exit
        if app.should_quit {
            break;
        }
    }
    tui.exit();

    Ok(())
}
