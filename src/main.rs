use crate::app::{App, Screen};

pub mod chat; 
pub mod app;
pub mod event;
pub mod ui;
pub mod config; 


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    
    let terminal = ratatui::init();

    let mut app = App::new(); 

    if let Some(config) = config::load(){
        if !(config.discord_token.is_empty()){
            app.discord_token = config.discord_token; 
            app.screen = Screen::Main;
        }
    }
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
