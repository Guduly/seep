use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, Screen, MenuItem};  // ← import Screen
use crate::chat::{Platform, Conversation}; 
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            Screen::Login => self.render_login(area, buf),
            Screen::Main => self.render_main(area, buf),
            Screen::Discord => self.render_discord(area,buf),
            Screen::Whatsapp => self.render_chat(area, buf, Some(Platform::Whatsapp)), 
            Screen::AllChats => self.render_chat(area, buf, Some(None)), 
        }
    }
}

impl App {
    fn render_login(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(area);

        let top_block = Block::bordered()
            .title("Seep")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        Paragraph::new("Enter your Discord User Token:")
            .block(top_block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered()
            .render(chunks[0], buf);

        let bottom_block = Block::bordered()
            .title("Token")
            .border_type(BorderType::Rounded);

        Paragraph::new(self.token_input.as_str())
            .block(bottom_block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .render(chunks[1], buf);
    }

    fn render_main(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area); 
        
        let platforms = [
            ("Chats", &MenuItem::ContinueChats), 
            ("Discord", &MenuItem::Discord), 
            ("Whatsapp", &MenuItem::Whatsapp), 
            ("Messages", &MenuItem::Messages), 
        ]; 
        
        for(i, (label, platform)) in platforms.iter().enumerate(){
            let selec = std::mem::discriminant(*platform)
                == std::mem::discriminant(&self.selected);
            
            let color = if selec {Color::Yellow} else {Color::Cyan};

            Paragraph::new(*label)
                .fg(color)
                .bg(Color::Black)
                .centered()
                .block(Block::bordered().border_type(BorderType::Rounded))
                .render(chunks[i], buf);
        }

    }

    fn render_discord(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Discord DMs Coming Soon!")
            .fg(Color::Green)
            .bg(Color::Black)
            .centered()
            .render(area, buf);
    }
}
