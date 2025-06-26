use warlords::network::MultiplayerServer;
use warlords::database::CharacterDatabase;
use std::path::PathBuf;
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("warlords-server")
        .about("Warlords Multiplayer Server")
        .version("0.1.0")
        .arg(Arg::new("port")
            .short('p')
            .long("port")
            .value_name("PORT")
            .help("Port to listen on")
            .default_value("2323"))
        .arg(Arg::new("database")
            .short('d')
            .long("database")
            .value_name("FILE")
            .help("Database file path")
            .default_value("characters.json"))
        .get_matches();

    let port: u16 = matches.get_one::<String>("port").unwrap().parse()?;
    let db_path = PathBuf::from(matches.get_one::<String>("database").unwrap());
    
    println!("🎮 Loading character database from: {:?}", db_path);
    let database = CharacterDatabase::load_or_create(&db_path)?;
    
    let server = MultiplayerServer::new(database);
    
    println!("🚀 Starting Warlords Multiplayer Server...");
    println!("🌐 Connect with: telnet localhost {}", port);
    println!("📡 Or share with ngrok: ngrok tcp {}", port);
    
    server.start(port).await?;
    
    Ok(())
}