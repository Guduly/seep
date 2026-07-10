use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader}; 
use tokio::net::UnixStream; 
use serde::Deserialize; 

#[derive(Deserialize, Debug, Clone)]
pub struct Contact{
    pub jid: String, 
    pub name: String,
}

pub async fn get_contacts() -> color_eyre::Result<Vec<Contact>>{
    let mut stream = UnixStream::connect("/tmp/seep-bridge.sock").await?; 
    stream.write_all(b"{\"action\":\"get_contacts\"}\n").await?; 

    let mut reader = BufReader::new(&mut stream); 
    let mut line = String::new(); 

    reader.read_line(&mut line).await?; 

    let contacts: Vec<Contact> = serde_json::from_str(&line)?;  
    
    Ok(contacts)
}

pub async fn send_message(jid: &str, text: &str) -> color_eyre::Result<()>{
    let mut stream = UnixStream::connect("/tmp/seep-bridge.sock").await?; 
    let cmd = format!("{{\"action\":\"send\",\"to\":\"{}\",\"text\":\"{}\"}}\n",
        jid, text); 

    stream.write_all(cmd.as_bytes()).await?; 
    
    Ok(())
}

