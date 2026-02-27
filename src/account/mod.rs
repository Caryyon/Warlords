use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

/// Account system for local user management
/// Stores accounts in ~/.local/share/warlords/accounts/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: chrono::DateTime<chrono::Utc>,
    pub characters: Vec<String>, // Character names owned by this account
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountManager {
    accounts_path: PathBuf,
    #[serde(skip)]
    current_account: Option<Account>,
}

impl AccountManager {
    pub fn new() -> Result<Self> {
        let accounts_path = Self::get_accounts_path()?;
        fs::create_dir_all(&accounts_path)?;

        Ok(Self {
            accounts_path,
            current_account: None,
        })
    }

    fn get_accounts_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        let path = PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("warlords")
            .join("accounts");
        Ok(path)
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn get_account_file(&self, username: &str) -> PathBuf {
        self.accounts_path.join(format!("{}.json", username.to_lowercase()))
    }

    pub fn account_exists(&self, username: &str) -> bool {
        self.get_account_file(username).exists()
    }

    pub fn signup(&mut self, username: &str, password: &str) -> Result<()> {
        // Validate username
        if username.len() < 3 {
            return Err(anyhow::anyhow!("Username must be at least 3 characters"));
        }
        if username.len() > 20 {
            return Err(anyhow::anyhow!("Username must be 20 characters or less"));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(anyhow::anyhow!("Username can only contain letters, numbers, and underscores"));
        }

        // Validate password
        if password.len() < 4 {
            return Err(anyhow::anyhow!("Password must be at least 4 characters"));
        }

        // Check if account exists
        if self.account_exists(username) {
            return Err(anyhow::anyhow!("Account '{}' already exists", username));
        }

        let now = chrono::Utc::now();
        let account = Account {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            created_at: now,
            last_login: now,
            characters: Vec::new(),
        };

        let account_file = self.get_account_file(username);
        let json = serde_json::to_string_pretty(&account)?;
        fs::write(&account_file, json)?;

        self.current_account = Some(account);
        Ok(())
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let account_file = self.get_account_file(username);

        if !account_file.exists() {
            return Err(anyhow::anyhow!("Account '{}' not found", username));
        }

        let json = fs::read_to_string(&account_file)?;
        let mut account: Account = serde_json::from_str(&json)?;

        // Verify password
        let password_hash = Self::hash_password(password);
        if account.password_hash != password_hash {
            return Err(anyhow::anyhow!("Incorrect password"));
        }

        // Update last login
        account.last_login = chrono::Utc::now();
        let json = serde_json::to_string_pretty(&account)?;
        fs::write(&account_file, json)?;

        self.current_account = Some(account);
        Ok(())
    }

    pub fn logout(&mut self) {
        self.current_account = None;
    }

    pub fn is_logged_in(&self) -> bool {
        self.current_account.is_some()
    }

    pub fn current_account(&self) -> Option<&Account> {
        self.current_account.as_ref()
    }

    pub fn current_username(&self) -> Option<&str> {
        self.current_account.as_ref().map(|a| a.username.as_str())
    }

    pub fn add_character_to_account(&mut self, character_name: &str) -> Result<()> {
        let account = self.current_account.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;

        if !account.characters.contains(&character_name.to_string()) {
            account.characters.push(character_name.to_string());
        }

        // Save updated account (get username before borrowing)
        let username = self.current_account.as_ref()
            .map(|a| a.username.clone())
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        let account_file = self.get_account_file(&username);
        let json = serde_json::to_string_pretty(&self.current_account)?;
        fs::write(&account_file, json)?;

        Ok(())
    }

    pub fn remove_character_from_account(&mut self, character_name: &str) -> Result<()> {
        let account = self.current_account.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;

        account.characters.retain(|c| c != character_name);

        // Save updated account (get username before borrowing)
        let username = self.current_account.as_ref()
            .map(|a| a.username.clone())
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        let account_file = self.get_account_file(&username);
        let json = serde_json::to_string_pretty(&self.current_account)?;
        fs::write(&account_file, json)?;

        Ok(())
    }

    pub fn get_account_characters(&self) -> Vec<String> {
        self.current_account
            .as_ref()
            .map(|a| a.characters.clone())
            .unwrap_or_default()
    }

    pub fn list_all_accounts(&self) -> Result<Vec<String>> {
        let mut accounts = Vec::new();

        if self.accounts_path.exists() {
            for entry in fs::read_dir(&self.accounts_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        accounts.push(stem.to_string());
                    }
                }
            }
        }

        accounts.sort();
        Ok(accounts)
    }
}

/// Starting occupations that affect character background and starting skills
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Occupation {
    Farmer,
    Blacksmith,
    Barkeep,
    Merchant,
    Fisher,
    StableHand,
    Miller,
    Hunter,
    Shepherd,
    Carpenter,
    Miner,
    Herbalist,
}

impl Occupation {
    pub fn all() -> Vec<Occupation> {
        vec![
            Occupation::Farmer,
            Occupation::Blacksmith,
            Occupation::Barkeep,
            Occupation::Merchant,
            Occupation::Fisher,
            Occupation::StableHand,
            Occupation::Miller,
            Occupation::Hunter,
            Occupation::Shepherd,
            Occupation::Carpenter,
            Occupation::Miner,
            Occupation::Herbalist,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Occupation::Farmer => "Farmer",
            Occupation::Blacksmith => "Blacksmith's Apprentice",
            Occupation::Barkeep => "Barkeep",
            Occupation::Merchant => "Merchant's Assistant",
            Occupation::Fisher => "Fisher",
            Occupation::StableHand => "Stable Hand",
            Occupation::Miller => "Miller",
            Occupation::Hunter => "Hunter",
            Occupation::Shepherd => "Shepherd",
            Occupation::Carpenter => "Carpenter",
            Occupation::Miner => "Miner",
            Occupation::Herbalist => "Herbalist",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Occupation::Farmer => "You've worked the fields since childhood. Strong back, simple life.",
            Occupation::Blacksmith => "Hammering steel taught you patience and precision.",
            Occupation::Barkeep => "Years of serving drinks gave you insight into human nature.",
            Occupation::Merchant => "Counting coins and bartering taught you the value of things.",
            Occupation::Fisher => "The river's edge was your home, patience your virtue.",
            Occupation::StableHand => "Horses and livestock were your companions.",
            Occupation::Miller => "Grinding grain is honest work that builds strong arms.",
            Occupation::Hunter => "The forest holds no secrets from you.",
            Occupation::Shepherd => "Long nights under the stars watching your flock.",
            Occupation::Carpenter => "You can build or fix almost anything with wood.",
            Occupation::Miner => "Deep in the earth, you learned to read stone.",
            Occupation::Herbalist => "Plants whisper their secrets to those who listen.",
        }
    }

    pub fn starting_skills(&self) -> Vec<(&'static str, u8)> {
        match self {
            Occupation::Farmer => vec![("Endurance", 15), ("Survival", 10)],
            Occupation::Blacksmith => vec![("Crafting", 15), ("Axe", 10)],
            Occupation::Barkeep => vec![("Persuasion", 15), ("Perception", 10)],
            Occupation::Merchant => vec![("Persuasion", 10), ("Perception", 15)],
            Occupation::Fisher => vec![("Survival", 15), ("Swimming", 10)],
            Occupation::StableHand => vec![("Animal Handling", 15), ("Riding", 10)],
            Occupation::Miller => vec![("Endurance", 15), ("Climbing", 5)],
            Occupation::Hunter => vec![("Tracking", 15), ("Bow", 10)],
            Occupation::Shepherd => vec![("Animal Handling", 10), ("Perception", 15)],
            Occupation::Carpenter => vec![("Crafting", 15), ("Climbing", 10)],
            Occupation::Miner => vec![("Endurance", 15), ("Search", 10)],
            Occupation::Herbalist => vec![("Plant ID", 15), ("Healing", 10)],
        }
    }

    pub fn starting_gold_bonus(&self) -> u32 {
        match self {
            Occupation::Merchant => 25,  // Merchants save more
            Occupation::Blacksmith => 15,
            Occupation::Barkeep => 10,
            Occupation::Hunter => 5,
            _ => 0,
        }
    }

    pub fn village_name(&self) -> &'static str {
        // Different occupations come from different villages
        match self {
            Occupation::Farmer | Occupation::Shepherd | Occupation::Miller => "Millbrook",
            Occupation::Blacksmith | Occupation::Miner | Occupation::Carpenter => "Ironhaven",
            Occupation::Fisher => "Riverside",
            Occupation::Hunter | Occupation::Herbalist => "Greenwood",
            Occupation::Barkeep | Occupation::Merchant | Occupation::StableHand => "Crossroads",
        }
    }

    pub fn tutorial_intro(&self) -> &'static str {
        match self {
            Occupation::Farmer => "The morning sun rises over the wheat fields of Millbrook, same as every day for as long as you can remember. But today feels different...",
            Occupation::Blacksmith => "The forge fire crackles as you pump the bellows in Ironhaven. Master Erikson is late, which never happens...",
            Occupation::Barkeep => "The tavern is quiet this morning in Crossroads. Too quiet. The regulars should be here by now...",
            Occupation::Merchant => "The caravan was supposed to arrive at Crossroads yesterday. Something is wrong...",
            Occupation::Fisher => "Your nets came up empty again in Riverside. The fish have fled downstream. Even the river seems afraid...",
            Occupation::StableHand => "The horses in the Crossroads stable are restless, whinnying and pawing at their stalls. They sense something...",
            Occupation::Miller => "The windmill's sails turn lazily over Millbrook. From up here, you can see the dust cloud approaching on the horizon...",
            Occupation::Hunter => "The forest near Greenwood has gone silent. No birdsong, no rustling in the undergrowth. Even the wind seems to hold its breath...",
            Occupation::Shepherd => "Your flock clusters together nervously on the hills above Millbrook. They won't graze, won't move. Something has spooked them...",
            Occupation::Carpenter => "The beam you're carving splits wrong in Ironhaven. A bad omen, the old folks would say...",
            Occupation::Miner => "Deep in the Ironhaven mines, you found something strange. Markings on the rock, ancient and glowing faintly...",
            Occupation::Herbalist => "The plants in your garden at Greenwood have wilted overnight, despite your care. Nature itself seems troubled...",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash() {
        let hash1 = AccountManager::hash_password("test123");
        let hash2 = AccountManager::hash_password("test123");
        let hash3 = AccountManager::hash_password("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_occupation_skills() {
        for occupation in Occupation::all() {
            let skills = occupation.starting_skills();
            assert!(!skills.is_empty(), "Occupation {:?} should have starting skills", occupation);
        }
    }
}
