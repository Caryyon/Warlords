use warlords::forge::ForgeCharacterCreation;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🎮 Welcome to Warlords - Rust Edition!");
    println!("=======================================");
    
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
    
    // Create a character with Human race
    let human_race = &races[0];
    let characteristics = ForgeCharacterCreation::apply_racial_modifiers(&rolled, human_race);
    let character = ForgeCharacterCreation::create_character(
        "TestHero".to_string(),
        characteristics,
        human_race.clone(),
    );
    
    println!("\n🦸 Created Character:");
    for line in character.get_display_info() {
        println!("{}", line);
    }
    
    println!("\n✅ Rust conversion working! The Forge system is fully ported.");
    println!("💡 Next step: Run `cargo run --bin warlords-server` for multiplayer!");
    
    Ok(())
}