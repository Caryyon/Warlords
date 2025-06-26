use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use warlords::forge::ForgeCharacterCreation;
use warlords::database::CharacterDatabase;
use anyhow::Result;
use clap::{Arg, Command};

#[derive(Clone)]
struct SimpleSession {
    authenticated: bool,
    character_name: Option<String>,
}

struct SimpleServer {
    sessions: Arc<Mutex<HashMap<String, SimpleSession>>>,
    database: Arc<Mutex<CharacterDatabase>>,
}

impl SimpleServer {
    fn new() -> Result<Self> {
        let database = CharacterDatabase::load_or_create(&PathBuf::from("characters.json"))?;
        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            database: Arc::new(Mutex::new(database)),
        })
    }

    async fn start(&self, port: u16) -> Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        println!("🎮 Warlords Simple Server running on port {}", port);
        println!("📡 Connect with: telnet localhost {}", port);
        println!("💡 Commands: help, create <name> <password>, login <name> <password>, stats, quit");
        
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("🔗 New connection from: {}", addr);
            
            let sessions = Arc::clone(&self.sessions);
            let database = Arc::clone(&self.database);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, sessions, database).await {
                    eprintln!("Error handling client {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_client(
        mut stream: TcpStream,
        sessions: Arc<Mutex<HashMap<String, SimpleSession>>>,
        database: Arc<Mutex<CharacterDatabase>>,
    ) -> Result<()> {
        let addr = stream.peer_addr()?.to_string();
        
        // Insert session
        {
            let mut sessions_lock = sessions.lock().await;
            sessions_lock.insert(addr.clone(), SimpleSession {
                authenticated: false,
                character_name: None,
            });
        }

        // Send welcome
        let welcome = format!("{}{}{}{}{}",
            "\x1b[2J\x1b[H", // Clear screen
            "\x1b[93m╔════════════════════════════════════╗\r\n",
            "║          🎮 WARLORDS 🎮            ║\r\n",
            "║    Rust-Powered RPG Server         ║\r\n",
            "╚════════════════════════════════════╝\x1b[0m\r\n\r\n",
        );
        stream.write_all(welcome.as_bytes()).await?;
        stream.write_all(b"Type 'help' for commands.\r\n> ").await?;

        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    let response = Self::handle_command(input, &addr, &sessions, &database).await;
                    
                    match response {
                        Ok(Some(msg)) => {
                            write_half.write_all(format!("{}\r\n> ", msg).as_bytes()).await?;
                        }
                        Ok(None) => break, // Quit command
                        Err(e) => {
                            let error_msg = format!("\x1b[91mError: {}\x1b[0m", e);
                            write_half.write_all(format!("{}\r\n> ", error_msg).as_bytes()).await?;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Clean up session
        {
            let mut sessions_lock = sessions.lock().await;
            sessions_lock.remove(&addr);
        }
        
        Ok(())
    }

    async fn handle_command(
        input: &str,
        addr: &str,
        sessions: &Arc<Mutex<HashMap<String, SimpleSession>>>,
        database: &Arc<Mutex<CharacterDatabase>>,
    ) -> Result<Option<String>> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Some("Invalid command. Type 'help' for available commands.".to_string()));
        }

        let command = parts[0].to_lowercase();
        
        match command.as_str() {
            "help" => {
                Ok(Some(format!("{}{}{}{}{}{}{}",
                    "\x1b[96m=== COMMANDS ===\x1b[0m\r\n",
                    "\x1b[93mcreate <name> <password>\x1b[0m - Create new character\r\n",
                    "\x1b[93mlogin <name> <password>\x1b[0m  - Login to character\r\n",
                    "\x1b[93mstats\x1b[0m                   - Show character stats\r\n",
                    "\x1b[93mroll\x1b[0m                    - Roll new characteristics\r\n",
                    "\x1b[93mlist\x1b[0m                    - List all characters\r\n",
                    "\x1b[93mquit\x1b[0m                    - Exit game"
                )))
            }
            "create" => {
                if parts.len() < 3 {
                    return Ok(Some("Usage: create <name> <password>".to_string()));
                }
                
                let name = parts[1];
                let password = parts[2];
                
                // Create character with random rolls
                let rolled = ForgeCharacterCreation::roll_characteristics();
                let races = ForgeCharacterCreation::get_available_races();
                let human_race = races[0].clone();
                let characteristics = ForgeCharacterCreation::apply_racial_modifiers(&rolled, &human_race);
                let character = ForgeCharacterCreation::create_character(
                    name.to_string(),
                    characteristics,
                    human_race,
                );

                // Save to database
                {
                    let mut db_lock = database.lock().await;
                    match db_lock.create_character(name.to_string(), password.to_string(), character) {
                        Ok(()) => {
                            let _ = db_lock.save(&PathBuf::from("characters.json"));
                            
                            // Update session
                            let mut sessions_lock = sessions.lock().await;
                            if let Some(session) = sessions_lock.get_mut(addr) {
                                session.authenticated = true;
                                session.character_name = Some(name.to_string());
                            }
                            
                            Ok(Some(format!("\x1b[92m✅ Character '{}' created and logged in!\x1b[0m", name)))
                        }
                        Err(e) => Ok(Some(format!("Failed to create character: {}", e)))
                    }
                }
            }
            "login" => {
                if parts.len() < 3 {
                    return Ok(Some("Usage: login <name> <password>".to_string()));
                }
                
                let name = parts[1];
                let password = parts[2];
                
                let result = {
                    let db_lock = database.lock().await;
                    db_lock.authenticate(name, password)
                };

                match result {
                    Ok(_character) => {
                        let mut sessions_lock = sessions.lock().await;
                        if let Some(session) = sessions_lock.get_mut(addr) {
                            session.authenticated = true;
                            session.character_name = Some(name.to_string());
                        }
                        Ok(Some(format!("\x1b[92m✅ Welcome back, {}!\x1b[0m", name)))
                    }
                    Err(_) => Ok(Some("\x1b[91m❌ Invalid credentials\x1b[0m".to_string()))
                }
            }
            "stats" => {
                let session_info = {
                    let sessions_lock = sessions.lock().await;
                    sessions_lock.get(addr).cloned()
                };

                if let Some(session) = session_info {
                    if session.authenticated {
                        if let Some(char_name) = &session.character_name {
                            let character_info = {
                                let db_lock = database.lock().await;
                                db_lock.authenticate(char_name, "").ok() // We'll improve auth later
                            };

                            if let Some(character) = character_info {
                                let info = character.get_display_info();
                                let mut result = String::from("\x1b[93m=== CHARACTER SHEET ===\x1b[0m\r\n");
                                for line in info {
                                    result.push_str(&format!("{}\r\n", line));
                                }
                                Ok(Some(result))
                            } else {
                                Ok(Some("Character not found.".to_string()))
                            }
                        } else {
                            Ok(Some("No character loaded.".to_string()))
                        }
                    } else {
                        Ok(Some("Please login first.".to_string()))
                    }
                } else {
                    Ok(Some("Session not found.".to_string()))
                }
            }
            "roll" => {
                let rolled = ForgeCharacterCreation::roll_characteristics();
                let mut result = String::from("\x1b[96m🎲 New Character Roll:\x1b[0m\r\n");
                result.push_str(&format!("Strength:  {:.1} ({})\r\n", rolled.strength.total, rolled.strength.formula));
                result.push_str(&format!("Stamina:   {:.1} ({})\r\n", rolled.stamina.total, rolled.stamina.formula));
                result.push_str(&format!("Intellect: {:.1} ({})\r\n", rolled.intellect.total, rolled.intellect.formula));
                result.push_str(&format!("Insight:   {:.1} ({})\r\n", rolled.insight.total, rolled.insight.formula));
                result.push_str(&format!("Dexterity: {:.1} ({})\r\n", rolled.dexterity.total, rolled.dexterity.formula));
                result.push_str(&format!("Awareness: {:.1} ({})\r\n", rolled.awareness.total, rolled.awareness.formula));
                result.push_str(&format!("Speed:     {} ({})\r\n", rolled.speed.total, rolled.speed.formula));
                result.push_str(&format!("Power:     {} ({})\r\n", rolled.power.total, rolled.power.formula));
                result.push_str(&format!("Luck:      {} ({})", rolled.luck.total, rolled.luck.formula));
                Ok(Some(result))
            }
            "list" => {
                let characters = {
                    let db_lock = database.lock().await;
                    db_lock.list_characters()
                };

                let mut result = String::from("\x1b[95m=== ALL CHARACTERS ===\x1b[0m\r\n");
                if characters.is_empty() {
                    result.push_str("No characters found.");
                } else {
                    for (name, last_played) in characters {
                        result.push_str(&format!("• {} (last played: {})\r\n", name, last_played.format("%Y-%m-%d %H:%M")));
                    }
                }
                Ok(Some(result))
            }
            "quit" | "exit" => {
                Ok(None) // Signal to close connection
            }
            _ => {
                Ok(Some("Unknown command. Type 'help' for available commands.".to_string()))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("warlords-simple-server")
        .about("Simple Warlords Multiplayer Server")
        .version("0.1.0")
        .arg(Arg::new("port")
            .short('p')
            .long("port")
            .value_name("PORT")
            .help("Port to listen on")
            .default_value("2323"))
        .get_matches();

    let port: u16 = matches.get_one::<String>("port").unwrap().parse()?;
    
    let server = SimpleServer::new()?;
    server.start(port).await?;
    
    Ok(())
}