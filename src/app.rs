use crate::chat::{Conversation, Message, Platform};
use crate::config;
use crate::event::{AppEvent, Event, EventHandler};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::DefaultTerminal;

#[derive(Debug)]
pub enum Screen {
    Login,
    Main,
    Discord,
    Whatsapp,
    Messages,
    AllChats,
}

#[derive(Debug)]
pub enum MenuItem {
    ContinueChats,
    Discord,
    Whatsapp,
    Messages,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub screen: Screen,
    pub token_input: String,
    pub discord_token: String,
    pub selected: MenuItem,
    pub conversations: Vec<Conversation>,
    pub active_conv: Option<usize>,
    pub message_input: String,
    pub contact_scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            screen: Screen::Login,
            token_input: String::new(),
            discord_token: String::new(),
            selected: MenuItem::ContinueChats,
            conversations: Vec::new(),
            active_conv: None,
            message_input: String::new(),
            contact_scroll: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        // start listening for incoming messages immediately
        self.start_message_listener();

        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event
                        && key_event.kind == crossterm::event::KeyEventKind::Press
                    {
                        self.handle_key_events(key_event)?;
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::ContactsLoaded(contacts) => {
                        self.conversations = contacts
                            .into_iter()
                            .map(|c| Conversation {
                                platform: Platform::Whatsapp,
                                user: c.name,
                                messages: vec![],
                                jid: c.jid,
                            })
                            .collect();
                        self.contact_scroll = 0;
                    }
                    AppEvent::MessageReceived {
                        from,
                        text,
                        timestamp,
                    } => {
                        if let Some(conv) = self.conversations.iter_mut().find(|c| c.jid == from) {
                            conv.messages.push(Message {
                                from_me: false,
                                content: text,
                                timestamp,
                            });
                        }
                    }
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.screen {
            Screen::Login => match key_event.code {
                KeyCode::Char(c) => self.token_input.push(c),
                KeyCode::Backspace => {
                    self.token_input.pop();
                }
                KeyCode::Esc => self.events.send(AppEvent::Quit),
                KeyCode::Enter if !self.token_input.is_empty() => {
                    config::save(&self.token_input)?;
                    self.discord_token = self.token_input.clone();
                    self.token_input.clear();
                    self.screen = Screen::Main;
                }
                _ => {}
            },
            Screen::Main => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Left | KeyCode::Char('j') => self.select_menu_item(true),
                KeyCode::Right | KeyCode::Char('k') => self.select_menu_item(false),
                KeyCode::Enter => self.select_current(),
                _ => {}
            },
            Screen::Discord => {
                if key_event.code == KeyCode::Esc {
                    self.screen = Screen::Main
                }
            }
            Screen::Whatsapp | Screen::AllChats => match key_event.code {
                KeyCode::Esc => self.screen = Screen::Main,
                KeyCode::Up => self.prev_conv(),
                KeyCode::Down => self.next_conv(),
                KeyCode::Enter => self.send_current_message(),
                KeyCode::Char(c) => self.message_input.push(c),
                KeyCode::Backspace => {
                    self.message_input.pop();
                }
                _ => {}
            },
            Screen::Messages => {
                if key_event.code == KeyCode::Esc {
                    self.screen = Screen::Main
                }
            }
        }
        Ok(())
    }

    pub fn next_conv(&mut self) {
        let max = self.conversations.len().saturating_sub(1);
        self.active_conv = Some(match self.active_conv {
            None => 0,
            Some(i) => (i + 1).min(max),
        });
    }

    pub fn prev_conv(&mut self) {
        self.active_conv = Some(match self.active_conv {
            None => 0,
            Some(i) => i.saturating_sub(1),
        });
    }

    pub fn select_menu_item(&mut self, reverse: bool) {
        self.selected = match self.selected {
            MenuItem::ContinueChats => {
                if reverse {
                    MenuItem::Messages
                } else {
                    MenuItem::Discord
                }
            }
            MenuItem::Discord => {
                if reverse {
                    MenuItem::ContinueChats
                } else {
                    MenuItem::Whatsapp
                }
            }
            MenuItem::Whatsapp => {
                if reverse {
                    MenuItem::Discord
                } else {
                    MenuItem::Messages
                }
            }
            MenuItem::Messages => {
                if reverse {
                    MenuItem::Whatsapp
                } else {
                    MenuItem::ContinueChats
                }
            }
        };
    }

    pub fn select_current(&mut self) {
        self.load_whatsapp_contacts();
        match self.selected {
            MenuItem::Discord => {
                if self.discord_token.is_empty() {
                    self.screen = Screen::Login;
                } else {
                    self.screen = Screen::Discord;
                }
            }
            MenuItem::Whatsapp => self.screen = Screen::Whatsapp,
            MenuItem::Messages => {}
            MenuItem::ContinueChats => self.screen = Screen::AllChats,
        }
    }

    pub fn send_current_message(&mut self) {
        if self.message_input.is_empty() {
            return;
        }
        let Some(idx) = self.active_conv else { return };
        let jid = self.conversations[idx].jid.clone();
        let text = self.message_input.clone();
        self.message_input.clear();

        // optimistic update — show immediately without waiting for confirmation
        self.conversations[idx].messages.push(Message {
            from_me: true,
            content: text.clone(),
            timestamp: "sending...".to_string(),
        });

        tokio::spawn(async move {
            if let Err(e) = crate::bridge::send_message(&jid, &text).await {
                eprintln!("Failed to send message: {e}");
            }
        });
    }

    pub fn load_whatsapp_contacts(&mut self) {
        let sender = self.events.sender.clone();
        tokio::spawn(async move {
            match crate::bridge::get_contacts().await {
                Ok(contacts) => {
                    let _ = sender.send(Event::App(AppEvent::ContactsLoaded(contacts)));
                }
                Err(e) => eprintln!("Failed to load contacts: {e}"),
            }
        });
    }

    pub fn start_message_listener(&mut self) {
        let sender = self.events.sender.clone();
        tokio::spawn(async move {
            crate::bridge::subscribe_messages(sender).await;
        });
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
