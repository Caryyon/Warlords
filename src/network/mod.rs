use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::forge::ForgeCharacter;
use crate::database::CharacterDatabase;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Login { name: String, password: String },
    CreateCharacter { name: String, password: String, character_data: String },
    GameAction { action: String, data: Option<String> },
    Chat { message: String },
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    LoginSuccess { character: ForgeCharacter },
    LoginFailed { reason: String },
    CharacterCreated { character: ForgeCharacter },
    CreationFailed { reason: String },
    GameUpdate { data: String },
    ChatMessage { from: String, message: String },
    SystemMessage { message: String },
    Error { message: String },
}

pub struct GameSession {
    pub id: Uuid,
    pub character: Option<ForgeCharacter>,
    pub authenticated: bool,
    pub tx: mpsc::UnboundedSender<ServerMessage>,
}

pub struct MultiplayerServer {
    sessions: Arc<Mutex<HashMap<Uuid, GameSession>>>,
    database: Arc<Mutex<CharacterDatabase>>,
}

impl MultiplayerServer {
    pub fn new(database: CharacterDatabase) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            database: Arc::new(Mutex::new(database)),
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        println!("🎮 Warlords Multiplayer Server running on port {}", port);
        println!("📡 Players can connect with: telnet localhost {}", port);
        
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
        sessions: Arc<Mutex<HashMap<Uuid, GameSession>>>,
        database: Arc<Mutex<CharacterDatabase>>,
    ) -> Result<()> {
        let session_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Create session
        {
            let mut sessions_lock = sessions.lock().await;
            sessions_lock.insert(session_id, GameSession {
                id: session_id,
                character: None,
                authenticated: false,
                tx: tx.clone(),
            });
        }

        // Send welcome message
        Self::send_welcome(&mut stream).await?;

        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        // Spawn task to handle outgoing messages
        let sessions_for_writer = Arc::clone(&sessions);
        let mut write_half = write_half;
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let formatted = Self::format_server_message(&message);
                if write_half.write_all(formatted.as_bytes()).await.is_err() {
                    break;
                }
            }
            
            // Clean up session when writer closes
            let mut sessions_lock = sessions_for_writer.lock().await;
            sessions_lock.remove(&session_id);
        });

        // Handle incoming messages
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

                    if let Err(e) = Self::handle_input(
                        input,
                        session_id,
                        &sessions,
                        &database,
                    ).await {
                        eprintln!("Error handling input: {}", e);
                    }
                }
                Err(_) => break,
            }
        }

        // Clean up session
        let mut sessions_lock = sessions.lock().await;
        sessions_lock.remove(&session_id);
        
        Ok(())
    }

    async fn send_welcome(stream: &mut TcpStream) -> Result<()> {
        let welcome = format!("{}{}{}",
            "\x1b[2J\x1b[H", // Clear screen and home cursor
            Self::create_welcome_screen(),
            "\r\nWelcome to Warlords! Type 'help' for commands.\r\n> "
        );
        stream.write_all(welcome.as_bytes()).await?;
        Ok(())
    }

    fn create_welcome_screen() -> String {
        format!("{}{}{}{}{}{}{}{}{}",
            "\x1b[93m", // Bright yellow
            "╔══════════════════════════════════════════════════════════════════════════════╗\r\n",
            "║                                  WARLORDS                                    ║\r\n",
            "║                        A Forge: Out of Chaos Adventure                      ║\r\n",
            "╚══════════════════════════════════════════════════════════════════════════════╝\r\n",
            "\x1b[96m", // Bright cyan
            "\r\nFrom humble farm worker to mighty warlord...\r\n",
            "Your destiny awaits in the realm of chaos!\r\n",
            "\x1b[0m" // Reset
        )
    }

    async fn handle_input(
        input: &str,
        session_id: Uuid,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
        database: &Arc<Mutex<CharacterDatabase>>,
    ) -> Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let command = parts[0].to_lowercase();
        
        match command.as_str() {
            "help" => {
                Self::send_help(session_id, sessions).await?;
            }
            "login" => {
                if parts.len() >= 3 {
                    let name = parts[1];
                    let password = parts[2];
                    Self::handle_login(session_id, name, password, sessions, database).await?;
                } else {
                    Self::send_error(session_id, "Usage: login <name> <password>", sessions).await?;
                }
            }
            "create" => {
                if parts.len() >= 3 {
                    let name = parts[1];
                    let password = parts[2];
                    Self::handle_create_character(session_id, name, password, sessions, database).await?;
                } else {
                    Self::send_error(session_id, "Usage: create <name> <password>", sessions).await?;
                }
            }
            "quit" | "exit" => {
                Self::send_system_message(session_id, "Goodbye!", sessions).await?;
            }
            _ => {
                // Check if user is authenticated for game commands
                let is_authenticated = {
                    let sessions_lock = sessions.lock().await;
                    sessions_lock.get(&session_id).map(|s| s.authenticated).unwrap_or(false)
                };

                if is_authenticated {
                    Self::handle_game_command(session_id, input, sessions).await?;
                } else {
                    Self::send_error(session_id, "Please login first. Type 'help' for commands.", sessions).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_login(
        session_id: Uuid,
        name: &str,
        password: &str,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
        database: &Arc<Mutex<CharacterDatabase>>,
    ) -> Result<()> {
        let result = {
            let db_lock = database.lock().await;
            db_lock.authenticate(name, password)
        };

        match result {
            Ok(character) => {
                // Update session
                {
                    let mut sessions_lock = sessions.lock().await;
                    if let Some(session) = sessions_lock.get_mut(&session_id) {
                        session.character = Some(character.clone());
                        session.authenticated = true;
                        let _ = session.tx.send(ServerMessage::LoginSuccess { character });
                    }
                }
                Self::send_system_message(session_id, &format!("Welcome back, {}!", name), sessions).await?;
            }
            Err(_) => {
                Self::send_error(session_id, "Invalid credentials", sessions).await?;
            }
        }

        Ok(())
    }

    async fn handle_create_character(
        session_id: Uuid,
        name: &str,
        password: &str,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
        database: &Arc<Mutex<CharacterDatabase>>,
    ) -> Result<()> {
        // For simplicity, create a basic character
        // In a full implementation, this would be a multi-step process
        use crate::forge::ForgeCharacterCreation;
        
        let rolled = ForgeCharacterCreation::roll_characteristics();
        let races = ForgeCharacterCreation::get_available_races();
        let human_race = races[0].clone(); // Default to human
        
        let characteristics = ForgeCharacterCreation::apply_racial_modifiers(&rolled, &human_race);
        let character = ForgeCharacterCreation::create_character(
            name.to_string(),
            characteristics,
            human_race,
        );

        let result = {
            let mut db_lock = database.lock().await;
            db_lock.create_character(name.to_string(), password.to_string(), character.clone())
        };

        match result {
            Ok(()) => {
                // Save database
                {
                    let db_lock = database.lock().await;
                    let _ = db_lock.save(&std::path::PathBuf::from("characters.json"));
                }

                // Update session
                {
                    let mut sessions_lock = sessions.lock().await;
                    if let Some(session) = sessions_lock.get_mut(&session_id) {
                        session.character = Some(character.clone());
                        session.authenticated = true;
                        let _ = session.tx.send(ServerMessage::CharacterCreated { character });
                    }
                }
                Self::send_system_message(session_id, &format!("Character {} created successfully!", name), sessions).await?;
            }
            Err(e) => {
                Self::send_error(session_id, &format!("Failed to create character: {}", e), sessions).await?;
            }
        }

        Ok(())
    }

    async fn handle_game_command(
        session_id: Uuid,
        input: &str,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
    ) -> Result<()> {
        match input.to_lowercase().as_str() {
            "stats" | "character" => {
                let character_info = {
                    let sessions_lock = sessions.lock().await;
                    if let Some(session) = sessions_lock.get(&session_id) {
                        session.character.as_ref().map(|c| c.get_display_info())
                    } else {
                        None
                    }
                };

                if let Some(info) = character_info {
                    Self::send_character_sheet(session_id, &info, sessions).await?;
                }
            }
            "look" => {
                Self::send_system_message(session_id, "You are in a simple starting area. More features coming soon!", sessions).await?;
            }
            _ => {
                Self::send_error(session_id, "Unknown command. Try 'stats', 'look', or 'help'", sessions).await?;
            }
        }

        Ok(())
    }

    async fn send_help(
        session_id: Uuid,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
    ) -> Result<()> {
        let help_text = format!("{}{}{}{}{}{}{}{}{}",
            "\x1b[96m", // Bright cyan
            "=== WARLORDS COMMANDS ===\r\n",
            "\x1b[93m", // Bright yellow
            "login <name> <password>  - Login to existing character\r\n",
            "create <name> <password> - Create new character\r\n",
            "stats                    - Show character stats\r\n",
            "look                     - Look around\r\n",
            "quit                     - Exit the game\r\n",
            "\x1b[0m" // Reset
        );

        {
            let sessions_lock = sessions.lock().await;
            if let Some(session) = sessions_lock.get(&session_id) {
                let _ = session.tx.send(ServerMessage::SystemMessage { message: help_text });
            }
        }

        Ok(())
    }

    async fn send_character_sheet(
        session_id: Uuid,
        info: &[String],
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
    ) -> Result<()> {
        let mut sheet = format!("{}=== CHARACTER SHEET ===\r\n", "\x1b[93m");
        for line in info {
            sheet.push_str(&format!("{}\r\n", line));
        }
        sheet.push_str("\x1b[0m");

        {
            let sessions_lock = sessions.lock().await;
            if let Some(session) = sessions_lock.get(&session_id) {
                let _ = session.tx.send(ServerMessage::SystemMessage { message: sheet });
            }
        }

        Ok(())
    }

    async fn send_system_message(
        session_id: Uuid,
        message: &str,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
    ) -> Result<()> {
        {
            let sessions_lock = sessions.lock().await;
            if let Some(session) = sessions_lock.get(&session_id) {
                let _ = session.tx.send(ServerMessage::SystemMessage { 
                    message: format!("\x1b[92m{}\x1b[0m\r\n> ", message) 
                });
            }
        }
        Ok(())
    }

    async fn send_error(
        session_id: Uuid,
        message: &str,
        sessions: &Arc<Mutex<HashMap<Uuid, GameSession>>>,
    ) -> Result<()> {
        {
            let sessions_lock = sessions.lock().await;
            if let Some(session) = sessions_lock.get(&session_id) {
                let _ = session.tx.send(ServerMessage::Error { 
                    message: format!("\x1b[91mError: {}\x1b[0m\r\n> ", message) 
                });
            }
        }
        Ok(())
    }

    fn format_server_message(message: &ServerMessage) -> String {
        match message {
            ServerMessage::SystemMessage { message } => message.clone(),
            ServerMessage::Error { message } => message.clone(),
            ServerMessage::LoginSuccess { .. } => {
                format!("\x1b[92mLogin successful!\x1b[0m\r\n> ")
            }
            ServerMessage::CharacterCreated { .. } => {
                format!("\x1b[92mCharacter created!\x1b[0m\r\n> ")
            }
            _ => format!("{}\r\n> ", serde_json::to_string(message).unwrap_or_default()),
        }
    }
}