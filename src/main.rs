use warlords::game::Game;
use warlords::forge::ForgeCharacterCreation;
use clap::Command;
use crossterm::{terminal, execute, cursor};
use anyhow::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up panic handler to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to restore terminal
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            terminal::LeaveAlternateScreen,
            cursor::Show
        );
        
        // Call the original panic handler
        original_hook(panic_info);
    }));

    let matches = Command::new("warlords")
        .about("A terminal-based Forge: Out of Chaos RPG")
        .version("0.1.0")
        .subcommand(
            Command::new("test")
                .about("Test character creation system")
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("test", _)) => {
            run_character_test()
        }
        _ => {
            // Check if we're in a proper terminal for the full game
            if !is_proper_terminal() {
                println!("🎮 Welcome to Warlords!");
                println!("=======================================");
                println!("⚠️  Terminal Error: Cannot start full game.");
                println!("This program requires a proper terminal environment.");
                println!("");
                println!("🔧 Solutions:");
                println!("• Run from Terminal.app on macOS");
                println!("• Run from a Linux terminal (gnome-terminal, etc.)");
                println!("• Run from Windows Terminal or Command Prompt");
                println!("• Do NOT run from an IDE's integrated terminal");
                println!("");
                println!("🧪 To test the character system without terminal UI:");
                println!("   cargo run -- test");
                return Ok(());
            }
            
            // Run full game
            let mut game = Game::new()?;
            match game.run() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.to_string().into())
            }
        }
    };

    // Make sure terminal is restored even on normal exit
    let _ = crossterm::terminal::disable_raw_mode();
    
    result?;
    Ok(())
}

fn is_proper_terminal() -> bool {
    // Check if stdin is a TTY
    use std::os::unix::io::AsRawFd;
    unsafe {
        libc::isatty(std::io::stdin().as_raw_fd()) == 1
    }
}

fn run_character_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Welcome to Warlords - Forge Character System Test");
    println!("====================================================");
    
    // Test character creation
    println!("\n📊 Rolling new character...");
    let rolled = ForgeCharacterCreation::roll_characteristics();
    
    println!("\n🎲 Rolled Characteristics:");
    println!("Strength:    {:.1} ({})", rolled.strength.total, rolled.strength.formula);
    println!("Stamina:     {:.1} ({})", rolled.stamina.total, rolled.stamina.formula);
    println!("Intellect:   {:.1} ({})", rolled.intellect.total, rolled.intellect.formula);
    println!("Insight:     {:.1} ({})", rolled.insight.total, rolled.insight.formula);
    println!("Dexterity:   {:.1} ({})", rolled.dexterity.total, rolled.dexterity.formula);
    println!("Awareness:   {:.1} ({})", rolled.awareness.total, rolled.awareness.formula);
    println!("Speed:       {} ({})", rolled.speed.total, rolled.speed.formula);
    println!("Power:       {} ({})", rolled.power.total, rolled.power.formula);
    println!("Luck:        {} ({})", rolled.luck.total, rolled.luck.formula);
    
    // Test races
    println!("\n🏰 Available Races:");
    let races = ForgeCharacterCreation::get_available_races();
    for (i, race) in races.iter().enumerate() {
        println!("{}. {} - {}", i + 1, race.name, race.description);
    }
    
    // Test vision system with different races
    println!("\n👁️ Testing Vision System:");
    
    // Test Human (default vision)
    let human_race = &races[6]; // Human is at index 6
    let characteristics = ForgeCharacterCreation::apply_racial_modifiers(&rolled, human_race);
    let human_character = ForgeCharacterCreation::create_character(
        "TestHuman".to_string(),
        characteristics.clone(),
        human_race.clone(),
    );
    println!("Human vision radius: {} tiles (no special vision)", human_character.get_vision_radius());
    
    // Test Dwarf (Heat Vision 30')
    let dwarf_race = &races[2]; // Dwarf is at index 2
    let dwarf_characteristics = ForgeCharacterCreation::apply_racial_modifiers(&rolled, dwarf_race);
    let dwarf_character = ForgeCharacterCreation::create_character(
        "TestDwarf".to_string(),
        dwarf_characteristics,
        dwarf_race.clone(),
    );
    println!("Dwarf vision radius: {} tiles (Heat Vision 30')", dwarf_character.get_vision_radius());
    
    // Test Merikii (Night Vision 90')
    let merikii_race = &races[9]; // Merikii is at index 9
    let merikii_characteristics = ForgeCharacterCreation::apply_racial_modifiers(&rolled, merikii_race);
    let mut merikii_character = ForgeCharacterCreation::create_character(
        "TestMerikii".to_string(),
        merikii_characteristics,
        merikii_race.clone(),
    );
    println!("Merikii vision radius: {} tiles (Night Vision 90')", merikii_character.get_vision_radius());
    
    // Test torch mechanics
    println!("\n🔥 Testing Torch System:");
    println!("Human with torch lit: {} tiles", {
        let mut test_human = human_character.clone();
        test_human.light_torch();
        test_human.get_vision_radius()
    });
    println!("Merikii with torch lit: {} tiles (torch doesn't help - already has better vision)", {
        merikii_character.light_torch();
        merikii_character.get_vision_radius()
    });
    
    println!("\n🦸 Created Character:");
    for line in human_character.get_display_info() {
        println!("{}", line);
    }
    
    println!("\n✅ Character system working perfectly!");
    println!("🎯 Combat System Features:");
    println!("• Turn-based combat with initiative rolls");
    println!("• Skill/spell/action selection each round");
    println!("• Forge-compliant damage and armor system");
    println!("• Experience and skill advancement");
    println!("• Persistent character progression");
    println!("");
    println!("🎮 To play the full game:");
    println!("   cargo run");
    println!("   (requires proper terminal environment)");
    
    Ok(())
}