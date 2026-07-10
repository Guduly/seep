use crate::event::{AppEvent, Event, EventHandler};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use crate::config; 
use crate::chat::{Conversation, Platform, Message}; 

#[derive(Debug)]
pub enum Screen{
    Login, 
    Main, 
    Discord, 
    Whatsapp, 
    Messages, 
    AllChats, 
}

/// Menu
#[derive(Debug)]
pub enum MenuItem{
    ContinueChats, 
    Discord, 
    Whatsapp, 
    Messages
}

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub counter: u8,
    pub events: EventHandler,
    pub screen: Screen, 
    pub token_input: String, 
    pub discord_token: String, 
    pub selected: MenuItem, 
    pub conversations: Vec<Conversation>, 
    pub active_conv: Option<usize>,
    pub message_input: String
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            screen: Screen::Login, 
            token_input: String::new(), 
            discord_token: String::new(), 
            selected: MenuItem::ContinueChats, 
            conversations: Vec::new(), 
            active_conv: None, 
            message_input: String::new(), 
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = Self::default();
       
        /*
            app.conversations = vec![
            Conversation {
                platform: Platform::Whatsapp,
                user: "Alice".to_string(),
                messages: vec![
                    Message { from_me: false, content: "Hey!".to_string(), timestamp: "10:00".to_string() },
                    Message { from_me: true,  content: "Hi Alice!".to_string(), timestamp: "10:01".to_string() },
                    Message { from_me: false, content: "How are you?".to_string(), timestamp: "10:02".to_string() },
                ],
                _ => {}
            },
            Conversation {
                platform: Platform::Discord,
                user: "Bob".to_string(),
                messages: vec![
                    Message { from_me: false, content: "yo".to_string(), timestamp: "11:00".to_string() },
                    Message { from_me: true,  content: "hey bob".to_string(), timestamp: "11:01".to_string() },
                ],
                
            },
        ];
        */
        app
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Quit => self.quit(),
                    AppEvent::ContactsLoaded(contacts) => {
                        self.conversations = contacts.iter().map(|c|{
                            Conversation{
                                platform: Platform::Whatsapp, 
                                user: c.name.clone(), 
                                messages: vec![],
                                jid : c.jid.clone(),
                            }
                        }).collect(); 
                        }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.screen{
            Screen::Login => match key_event.code{
                KeyCode::Char(c) => self.token_input.push(c), 
                KeyCode::Backspace => {self.token_input.pop(); }, 
                KeyCode::Esc => self.events.send(AppEvent::Quit),
                KeyCode::Enter => {
                    if !self.token_input.is_empty(){
                        config::save(&self.token_input)?;
                        self.discord_token = self.token_input.clone();
                        self.token_input = "".to_string(); 
                        self.screen = Screen::Main;
                    }
                },
                // Other handlers you could add here.
                _ => {}
            },
            Screen::Main => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Left | KeyCode::Char('j') => self.select_menu_item(true),
                KeyCode::Right | KeyCode::Char('k') => self.select_menu_item(false),
                KeyCode::Enter => self.select_current(),
                _ => {}
            }, 
            Screen::Discord => match key_event.code{
                KeyCode::Esc => self.screen = Screen::Main, 
                _ => {}
            }, 
            Screen::Whatsapp | Screen::AllChats => match key_event.code{
                KeyCode::Esc => self.screen = Screen::Main,
                KeyCode::Up | KeyCode::Char('k') => self.prev_conv(),
                KeyCode::Down | KeyCode::Char('j') => self.next_conv(),
                KeyCode::Char(c) => self.message_input.push(c), 
                KeyCode::Backspace => {self.message_input.pop();}, 
                _ => {}
            },
            Screen::Messages => match key_event.code{
                KeyCode::Esc => self.screen = Screen::Main, 
                _ => {}
            }, 
        }
        Ok(())
    }
   
    pub fn next_conv(&mut self){
        let max = self.conversations.len().saturating_sub(1); 
        self.active_conv = Some( match self.active_conv{
            None => 0, 
            Some(i) => (i+1).min(max), 
        }); 
    }
    
    pub fn prev_conv(&mut self){
        self.active_conv = Some( match self.active_conv{
            None => 0, 
            Some(i) => i.saturating_sub(1), 
        }); 
    }

    /// Order { CC, Discord, Whatsapp, Messages}
    pub fn select_menu_item(&mut self, reverse: bool){
        self.selected = match self.selected{
            MenuItem::ContinueChats => {
                if reverse {MenuItem::Messages} 
                else {MenuItem::Discord} 
            },
            MenuItem::Discord => {
                if reverse
                {MenuItem::ContinueChats} 
                else {MenuItem::Whatsapp} 
            },
            MenuItem::Whatsapp => {
                if reverse 
                {MenuItem::Discord} 
                else {MenuItem::Messages} 
            },
            MenuItem::Messages => {
                if reverse 
                {MenuItem::Whatsapp}
                else {MenuItem::ContinueChats} 
            }, 
        };
    }
    
    pub fn select_current(&mut self){
        match self.selected{
            MenuItem::Discord => {
                if self.discord_token.is_empty() {self.screen = Screen::Login;}
                else {self.screen = Screen::Discord;} 
            },
            MenuItem::Whatsapp => {
                self.screen = Screen::Whatsapp;
                self.load_whatsapp_contacts();
            }
            MenuItem::Messages => {}, 
            MenuItem::ContinueChats => self.screen = Screen::AllChats, 
            _ => {}
        }
    }

    pub fn load_whatsapp_contacts(&mut self){
        let sender = self.events.sender.clone();

        tokio::spawn(async move {
            match crate::bridge::get_contacts().await{
                Ok(contacts) => {
                    let _ = sender.send(
                        crate::event::Event::App(crate::event::AppEvent::ContactsLoaded(contacts))
                        ); 
                }, 
                Err(e) => eprintln!("Failed to load contacts: {}", e),
            }
        }); 
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
} 
