use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
    text::{Line, Span}, 
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
            Screen::AllChats => self.render_chat(area, buf, None),
            Screen::Messages => {}, 
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
    
    fn render_chat(&self, area: Rect, buf: &mut Buffer, platform:Option<Platform>) {
        let convs: Vec<&Conversation> = match &platform{
            None => self.conversations.iter().collect(),
            Some(i) => self.conversations.iter().filter(|c| &c.platform == i).collect(), 
        }; 

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .split(area); 

        let contacts: Vec<Line> = if convs.is_empty() {vec![Line::from("No Contacts yet.")]}
        else{
           convs.iter().enumerate().map(|(i, c)| {
               let color = match &c.platform{
                   Platform::Whatsapp => Color::Green, 
                   Platform::Messages => Color::LightBlue, 
                   Platform::Discord => Color::Rgb(88, 101, 242),
               };

               let prefix = if Some(i) == self.active_conv {
                   "> "
               } else {" "}; 

               Line::from(Span::styled(
                       format!("{}{}", prefix, c.user), 
                       Style::default().fg(color),
                       ))
            }).collect() 
        };

        Paragraph::new(contacts)
            .block(
                Block::bordered()
                    .title("Chats")
                    .border_type(BorderType::Rounded)
            )
            .bg(Color::Black)
            .render(chunks[0], buf);
        
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(85),
                Constraint::Percentage(15),
            ])
            .split(chunks[1]);

        let message_lines: Vec<Line> = match self.active_conv{
            None => vec![Line::from(Span::styled(
                    "Select a Chat.",
                    Style::default().fg(Color::DarkGray), 
                    ))], 
            Some(i) => {
                let conv = &self.conversations[i];
                conv.messages.iter().map(|m|{
                    if m.from_me{
                        Line::from(Span::styled(
                        format!("{} you: {}", m.timestamp, m.content),
                        Style::default().fg(Color::Cyan), 
                        ))
                    }else{
                        Line::from(Span::styled(
                        format!("{} {}: {}", m.timestamp, conv.user, m.content),
                        Style::default().fg(Color::Red), 
                        ))
                    }
                }).collect()
            }
        };

        Paragraph::new(message_lines)
            .block(
                Block::bordered()
                    .title("Messages")
                    .border_type(BorderType::Rounded)
            )
            .bg(Color::Black)
            .render(right_chunks[0], buf);

        // input box
        Paragraph::new(self.message_input.as_str())
            .block(
                Block::bordered()
                    .title("Type a message...")
                    .border_type(BorderType::Rounded)
            )
            .fg(Color::Yellow)
            .bg(Color::Black)
            .render(right_chunks[1], buf);

    }

    fn render_discord(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Discord DMs Coming Soon!")
            .fg(Color::Green)
            .bg(Color::Black)
            .centered()
            .render(area, buf);
    }
}
