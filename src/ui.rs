use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect, Constraint, Direction, Layout},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical) 
            .constraints([
                Constraint::Percentage(80), 
                Constraint::Percentage(20), 
            ])
            .split(area); 
        
        let block = Block::bordered()
            .title("Seep")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        
        let text = format!(
            "Enter Discord User Token\n"
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        paragraph.render(chunks[0], buf);

        let block_bottom = Block::bordered()
            .title("Token")
           // .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        Paragraph::new(self.token_input.as_str())
            .block(block_bottom)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered()
            .render(chunks[1], buf);
    }
}
