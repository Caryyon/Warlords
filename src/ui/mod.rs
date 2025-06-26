use crossterm::{
    event::{self, Event, KeyEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use crate::forge::{RolledCharacteristics, ForgeRace};

pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

pub struct GameUI {
    terminal: TerminalType,
}

#[derive(Debug, Clone)]
pub enum UIState {
    Welcome,
    MainMenu,
    CharacterLogin,
    CharacterCreation(CharacterCreationState),
    CharacterList(Vec<(String, chrono::DateTime<chrono::Utc>)>, Option<usize>), // characters, selected_index
    Playing,
    CharacterMenu,
    WorldExploration(WorldExplorationState),
    DungeonExploration(DungeonExplorationState),
    Combat(CombatState),
}

#[derive(Debug, Clone)]
pub struct WorldExplorationState {
    pub current_zone: crate::world::ZoneCoord,
    pub player_local_pos: crate::world::LocalCoord,
    pub zone_data: Option<crate::world::WorldZone>,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DungeonExplorationState {
    pub dungeon: crate::world::DungeonLayout,
    pub player_pos: crate::world::LocalCoord,
    pub messages: Vec<String>,
    pub turn_count: u32,
}

#[derive(Debug, Clone)]
pub struct CombatState {
    pub encounter: crate::forge::CombatEncounter,
    pub selected_action: Option<usize>,
    pub available_skills: Vec<String>,
    pub selected_skill: Option<String>,
    pub combat_phase: CombatPhase,
    pub return_to_dungeon: Option<DungeonExplorationState>,
    pub current_skill_index: usize,
    pub skill_list_offset: usize, // For scrolling through long lists
}

#[derive(Debug, Clone)]
pub enum CombatPhase {
    InitiativeRoll,        // Rolling initiative for all participants
    DeclaringActions,      // All participants declare their actions
    SelectingSkill,        // Player selecting skill/spell/action
    SelectingTarget,       // Player selecting target for action
    ResolvingActions,      // Executing all declared actions
    RoundComplete,         // Round finished, preparing for next
    CombatComplete(bool),  // Combat over, true if player won
}

#[derive(Debug, Clone)]
pub struct CharacterCreationState {
    pub step: CreationStep,
    pub rolled_data: Option<RolledCharacteristics>,
    pub selected_race: Option<ForgeRace>,
    pub character_name: Option<String>,
    pub selected_skills: Vec<String>,
    pub available_skill_points: u8,
    pub selected_spells: Vec<(String, crate::forge::magic::MagicSchool)>,
    pub available_spell_picks: u8,
    pub selected_gear: Vec<String>,
    pub current_selection_index: usize,
    // Available options for UI display
    pub available_skills_list: Vec<String>,
    pub available_spells_list: Vec<(String, crate::forge::magic::MagicSchool)>,
    pub available_gear_list: Vec<(String, u32)>, // (item name, cost in gold)
    pub starting_gold: u32,
    pub spent_gold: u32,
}

#[derive(Debug, Clone)]
pub enum CreationStep {
    Rolling,
    RaceSelection,
    NameEntry,
    SkillSelection,
    SpellSelection,
    GearSelection,
    Confirmation,
}

impl GameUI {
    pub fn new() -> anyhow::Result<Self> {
        // Try to enable raw mode with better error handling
        terminal::enable_raw_mode()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to enable raw mode: {}.\n\nThis usually means you're not running in a proper terminal.\n\
                    Please try running the game from:\n\
                    - Terminal.app on macOS\n\
                    - A Linux terminal (gnome-terminal, konsole, etc.)\n\
                    - Windows Terminal or Command Prompt\n\
                    - NOT from an IDE's integrated terminal", 
                    e
                )
            })?;
        
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)
            .map_err(|e| anyhow::anyhow!("Failed to setup terminal screen: {}", e))?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| anyhow::anyhow!("Failed to create terminal: {}", e))?;
        
        // Clear the terminal to remove any previous content
        terminal.clear()
            .map_err(|e| anyhow::anyhow!("Failed to clear terminal: {}", e))?;
        
        Ok(GameUI { terminal })
    }

    pub fn cleanup(&mut self) -> anyhow::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show)?;
        Ok(())
    }

    pub fn draw(&mut self, state: &UIState, input_buffer: &str, current_character: Option<&crate::forge::ForgeCharacter>) -> anyhow::Result<()> {
        let state_clone = state.clone();
        let input_clone = input_buffer.to_string();
        let character_clone = current_character.cloned();
        self.terminal.draw(move |f| {
            match &state_clone {
                UIState::Welcome => Self::draw_welcome_static(f),
                UIState::MainMenu => Self::draw_main_menu_static(f, character_clone.as_ref()),
                UIState::CharacterLogin => Self::draw_character_login_static(f, &input_clone),
                UIState::CharacterCreation(creation_state) => Self::draw_character_creation_static(f, creation_state, &input_clone),
                UIState::CharacterList(character_list, selected_index) => Self::draw_character_list_static(f, Some(character_list), *selected_index),
                UIState::Playing => Self::draw_game_static(f, character_clone.as_ref()),
                UIState::CharacterMenu => Self::draw_character_menu_static(f, character_clone.as_ref()),
                UIState::WorldExploration(world_state) => Self::draw_world_exploration_static(f, world_state, character_clone.as_ref()),
                UIState::DungeonExploration(dungeon_state) => Self::draw_dungeon_exploration_static(f, dungeon_state, character_clone.as_ref()),
                UIState::Combat(combat_state) => Self::draw_combat_static(f, combat_state),
            }
        })?;
        Ok(())
    }

    fn draw_welcome_static(f: &mut Frame) {
        let area = f.size();
        
        // Create beautiful ASCII art title
        let title_art = vec![
            "██╗    ██╗ █████╗ ██████╗ ██╗      ██████╗ ██████╗ ██████╗ ███████╗",
            "██║    ██║██╔══██╗██╔══██╗██║     ██╔═══██╗██╔══██╗██╔══██╗██╔════╝",
            "██║ █╗ ██║███████║██████╔╝██║     ██║   ██║██████╔╝██║  ██║███████╗",
            "██║███╗██║██╔══██║██╔══██╗██║     ██║   ██║██╔══██╗██║  ██║╚════██║",
            "╚███╔███╔╝██║  ██║██║  ██║███████╗╚██████╔╝██║  ██║██████╔╝███████║",
            " ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝",
        ];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        let title_lines: Vec<Line> = title_art.iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::Yellow))))
            .collect();
        
        let title = Paragraph::new(title_lines)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[1]);

        // Subtitle
        let subtitle = Paragraph::new("A Forge: Out of Chaos Adventure")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
        f.render_widget(subtitle, chunks[2]);

        // Story intro
        let story = Paragraph::new(vec![
            Line::from("From humble farm worker to mighty warlord,"),
            Line::from("your destiny awaits in the realm of chaos!"),
            Line::from(""),
            Line::from(Span::styled("Press any key to continue...", Style::default().fg(Color::Green))),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
        f.render_widget(story, chunks[4]);
    }

    fn draw_main_menu_static(f: &mut Frame, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Title - show character info if logged in
        let title_text = if let Some(character) = current_character {
            format!("WARLORDS MAIN MENU - Playing as {}", character.name)
        } else {
            "WARLORDS MAIN MENU".to_string()
        };
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, chunks[0]);

        // Menu options - different based on whether character is logged in
        let menu_items = if current_character.is_some() {
            vec![
                ListItem::new("1. Return to Game World"),
                ListItem::new("2. Explore the World"),
                ListItem::new("3. Character Menu"),
                ListItem::new("4. Logout & Switch Character"),
                ListItem::new("5. Quit"),
                ListItem::new(""),
                ListItem::new(Span::styled("Select an option (1-5):", Style::default().fg(Color::Green))),
            ]
        } else {
            vec![
                ListItem::new("1. Login to Existing Character"),
                ListItem::new("2. Create New Character"),
                ListItem::new("3. List Characters"),
                ListItem::new("4. Quit"),
                ListItem::new(""),
                ListItem::new(Span::styled("Select an option (1-4):", Style::default().fg(Color::Green))),
            ]
        };

        let menu = List::new(menu_items)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)))
            .style(Style::default().fg(Color::White));
        f.render_widget(menu, chunks[1]);

        // Instructions
        let instructions = if current_character.is_some() {
            Paragraph::new("Enter your choice and press ENTER | M: Back to Game | Q/Ctrl+C: Quit")
        } else {
            Paragraph::new("Enter your choice and press ENTER | Q/Ctrl+C: Quit")
        };
        let instructions = instructions
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(instructions, chunks[2]);
    }

    fn draw_character_login_static(f: &mut Frame, input_buffer: &str) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let title = Paragraph::new("CHARACTER LOGIN")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(title, chunks[0]);

        let mut content_lines = vec![
            Line::from(""),
            Line::from("Enter character name and password"),
            Line::from(Span::styled("Format: name:password", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("Example: Aldric:mypassword", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Type 'back' to return to main menu", Style::default().fg(Color::Green))),
            Line::from(""),
            Line::from("Character login: "),
        ];

        // Add input line
        let input_line = if input_buffer.is_empty() {
            Line::from(Span::styled("▶ _", Style::default().fg(Color::Yellow)))
        } else {
            Line::from(vec![
                Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                Span::styled(input_buffer, Style::default().fg(Color::White)),
                Span::styled("_", Style::default().fg(Color::Yellow)),
            ])
        };
        content_lines.push(input_line);

        let content = Paragraph::new(content_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)));
        f.render_widget(content, chunks[1]);
    }

    fn draw_character_creation_static(f: &mut Frame, creation_state: &CharacterCreationState, input_buffer: &str) {
        
        match creation_state.step {
            CreationStep::Rolling => Self::draw_characteristic_rolling_static(f, creation_state),
            CreationStep::RaceSelection => Self::draw_race_selection_static(f),
            CreationStep::NameEntry => Self::draw_name_entry_static(f, creation_state, input_buffer),
            CreationStep::SkillSelection => Self::draw_skill_selection_static(f, creation_state),
            CreationStep::SpellSelection => Self::draw_spell_selection_static(f, creation_state),
            CreationStep::GearSelection => Self::draw_gear_selection_static(f, creation_state),
            CreationStep::Confirmation => Self::draw_character_confirmation_static(f, creation_state),
        }
    }

    fn draw_characteristic_rolling_static(f: &mut Frame, creation_state: &CharacterCreationState) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(chunks[0]);

        // Title
        let title = Paragraph::new("Forge: Out of Chaos - Character Creation")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, left_chunks[0]);

        // Main content
        let content = if let Some(rolled_data) = &creation_state.rolled_data {
            vec![
                Line::from(Span::styled("Your Rolled Characteristics:", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(format!("Strength:    {:.1} ({})", rolled_data.strength.total, rolled_data.strength.formula)),
                Line::from(format!("Stamina:     {:.1} ({})", rolled_data.stamina.total, rolled_data.stamina.formula)),
                Line::from(format!("Intellect:   {:.1} ({})", rolled_data.intellect.total, rolled_data.intellect.formula)),
                Line::from(format!("Insight:     {:.1} ({})", rolled_data.insight.total, rolled_data.insight.formula)),
                Line::from(format!("Dexterity:   {:.1} ({})", rolled_data.dexterity.total, rolled_data.dexterity.formula)),
                Line::from(format!("Awareness:   {:.1} ({})", rolled_data.awareness.total, rolled_data.awareness.formula)),
                Line::from(format!("Speed:       {} ({})", rolled_data.speed.total, rolled_data.speed.formula)),
                Line::from(format!("Power:       {} ({})", rolled_data.power.total, rolled_data.power.formula)),
                Line::from(format!("Luck:        {} ({})", rolled_data.luck.total, rolled_data.luck.formula)),
                Line::from(""),
                Line::from(Span::styled("Press C to continue or R to re-roll", Style::default().fg(Color::Green))),
            ]
        } else {
            vec![
                Line::from(Span::styled("Welcome to Forge: Out of Chaos Character Creation!", Style::default().fg(Color::Yellow))),
                Line::from(""),
                Line::from("In this step, you will roll dice to determine your character's"),
                Line::from("nine basic characteristics. These define your character's"),
                Line::from("natural abilities and potential."),
                Line::from(""),
                Line::from(Span::styled("Press ENTER to roll your characteristics", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("Rolling Method:"),
                Line::from("• First 6 characteristics: 2d6 + 1d10 (decimal)"),
                Line::from("• Speed: 1d4 + 1"),
                Line::from("• Power: 2d10"),
                Line::from("• Luck: 2d6 + 4"),
                Line::from(""),
                Line::from(Span::styled("Note: If you roll 0 on d10, it counts as 1.0", Style::default().fg(Color::Yellow))),
            ]
        };

        let main_content = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Step 1: Roll Characteristics").border_style(Style::default().fg(Color::Green)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(main_content, left_chunks[1]);

        // Instructions panel
        let instructions = Paragraph::new(vec![
            Line::from("Roll dice to determine your"),
            Line::from("character's basic abilities"),
            Line::from(""),
            Line::from("Roll 2d6 + 1d10 for each"),
            Line::from("of the first six characteristics"),
            Line::from(""),
            Line::from("If you roll 0 on the d10,"),
            Line::from("count it as 1.0 (full point)"),
            Line::from(""),
            Line::from("Roll 1d4+1 for Speed"),
            Line::from("Roll 2d10 for Power"),
            Line::from("Roll 2d6+4 for Luck"),
        ])
        .block(Block::default().borders(Borders::ALL).title("Instructions").border_style(Style::default().fg(Color::Cyan)))
        .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(instructions, chunks[1]);

        // Navigation
        let navigation = Paragraph::new("ENTER: Roll Characteristics | ESC: Cancel")
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Navigation").border_style(Style::default().fg(Color::Magenta)));
        f.render_widget(navigation, left_chunks[2]);
    }

    fn draw_race_selection_static(f: &mut Frame) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);
            
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(chunks[0]);
        
        // Title
        let title = Paragraph::new("Forge: Out of Chaos - Race Selection")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, left_chunks[0]);
        
        // Race list
        let races = vec![
            Line::from(Span::styled("Select Your Race:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("1. Berserker", Style::default().fg(Color::Red))),
            Line::from("   Large warriors who fear magic"),
            Line::from(Span::styled("2. Dunnar", Style::default().fg(Color::Magenta))),
            Line::from("   Pale beings with mind protection"),
            Line::from(Span::styled("3. Dwarf", Style::default().fg(Color::Yellow))),
            Line::from("   Stout warriors with heat vision"),
            Line::from(Span::styled("4. Elf", Style::default().fg(Color::Green))),
            Line::from("   Graceful beings with magical affinity"),
            Line::from(Span::styled("5. Ghantu", Style::default().fg(Color::Red))),
            Line::from("   Massive one-eyed humanoids"),
            Line::from(Span::styled("6. Higmoni", Style::default().fg(Color::Yellow))),
            Line::from("   Boar-like with fast healing"),
            Line::from(Span::styled("7. Human", Style::default().fg(Color::White))),
            Line::from("   Versatile with no penalties"),
            Line::from(Span::styled("8. Jher-em", Style::default().fg(Color::Cyan))),
            Line::from("   Small telepathic beings"),
            Line::from(Span::styled("9. Kithsara", Style::default().fg(Color::Green))),
            Line::from("   Lizard-like with natural armor"),
            Line::from(Span::styled("0. Merikii", Style::default().fg(Color::Yellow))),
            Line::from("   Bird-like dual wielders"),
            Line::from(Span::styled("#. Sprite", Style::default().fg(Color::Magenta))),
            Line::from("   Tiny empathic beings"),
        ];
        
        let race_list = Paragraph::new(races)
            .block(Block::default().borders(Borders::ALL).title("Step 2: Choose Race").border_style(Style::default().fg(Color::Green)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(race_list, left_chunks[1]);
        
        // Race details panel
        let details = vec![
            Line::from("Race Details:"),
            Line::from(""),
            Line::from("Each race has unique:"),
            Line::from("• Characteristic modifiers"),
            Line::from("• Maximum strength limits"),
            Line::from("• Special abilities"),
            Line::from("• Vision types"),
            Line::from("• Starting skills"),
            Line::from(""),
            Line::from("Some races have penalties:"),
            Line::from("• Berserkers cannot use magic"),
            Line::from("• Dunnar take sun damage"),
            Line::from("• Ghantu have learning disabilities"),
            Line::from("• Some have thin blood"),
            Line::from(""),
            Line::from("Choose wisely!"),
        ];
        
        let details_panel = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Race Information").border_style(Style::default().fg(Color::Cyan)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(details_panel, chunks[1]);
        
        // Navigation
        let navigation = Paragraph::new("1-9, 0, #: Select Race | ESC: Go Back")
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Navigation").border_style(Style::default().fg(Color::Magenta)));
        f.render_widget(navigation, left_chunks[2]);
    }

    fn draw_name_entry_static(f: &mut Frame, _creation_state: &CharacterCreationState, input_buffer: &str) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Forge: Out of Chaos - Character Name")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, chunks[0]);

        // Name entry content
        let mut content = vec![
            Line::from(Span::styled("Step 3: Enter Your Character's Name", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Your character needs a name to be known by in the world."),
            Line::from("This will be used for login and display throughout the game."),
            Line::from(""),
            Line::from(Span::styled("Requirements:", Style::default().fg(Color::Cyan))),
            Line::from("• Must be at least 2 characters long"),
            Line::from("• Can contain letters, numbers, and basic symbols"),
            Line::from("• Should be unique and memorable"),
            Line::from(""),
            Line::from(Span::styled("Enter your character's name:", Style::default().fg(Color::Green))),
            Line::from(""),
        ];

        // Add the input line with current buffer
        let input_line = if input_buffer.is_empty() {
            Line::from(vec![
                Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                Span::styled("_", Style::default().fg(Color::DarkGray)),
            ])
        } else {
            let color = if input_buffer.len() >= 2 { Color::Green } else { Color::Red };
            Line::from(vec![
                Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                Span::styled(input_buffer, Style::default().fg(color)),
                Span::styled("_", Style::default().fg(Color::Yellow)),
            ])
        };
        content.push(input_line);

        // Add status line
        let status_text = if input_buffer.is_empty() {
            "Start typing your character's name..."
        } else if input_buffer.len() < 2 {
            "Name must be at least 2 characters long"
        } else {
            "Press ENTER to continue"
        };
        let status_color = if input_buffer.len() >= 2 { Color::Green } else { Color::Red };
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(status_text, Style::default().fg(status_color))));

        let name_entry = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Character Naming").border_style(Style::default().fg(Color::Green)))
            .alignment(Alignment::Left);
        f.render_widget(name_entry, chunks[1]);

        // Navigation
        let navigation = Paragraph::new("Type name and press ENTER (min 2 chars) | ESC: Go Back")
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Navigation").border_style(Style::default().fg(Color::Magenta)));
        f.render_widget(navigation, chunks[2]);
    }

    fn draw_skill_selection_static(f: &mut Frame, creation_state: &CharacterCreationState) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        // Title
        let title = Paragraph::new("🎯 Skill Selection")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let skill_items: Vec<ListItem> = creation_state.available_skills_list.iter().enumerate().map(|(i, skill)| {
            let selected = creation_state.selected_skills.contains(skill);
            let is_current = i == creation_state.current_selection_index;
            
            let style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            
            let prefix = if selected { "✓ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, skill)).style(style)
        }).collect();

        let skills_list = List::new(skill_items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Available Skills (Points remaining: {})", 
                creation_state.available_skill_points
            )));
        f.render_widget(skills_list, chunks[1]);

        // Navigation
        let navigation = Paragraph::new("↑/↓: Navigate | Enter: Select/Deselect | C: Continue | Esc: Back")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(navigation, chunks[2]);
    }

    fn draw_spell_selection_static(f: &mut Frame, creation_state: &CharacterCreationState) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        // Title
        let title = Paragraph::new("🔮 Spell Selection")
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let spell_items: Vec<ListItem> = creation_state.available_spells_list.iter().enumerate().map(|(i, (spell, school))| {
            let selected = creation_state.selected_spells.iter().any(|(s, _)| s == spell);
            let is_current = i == creation_state.current_selection_index;
            
            let style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            
            let prefix = if selected { "✓ " } else { "  " };
            ListItem::new(format!("{}{} ({})", prefix, spell, school)).style(style)
        }).collect();

        let spells_list = List::new(spell_items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Available Spells (Picks remaining: {})", 
                creation_state.available_spell_picks
            )));
        f.render_widget(spells_list, chunks[1]);

        // Navigation
        let navigation = Paragraph::new("↑/↓: Navigate | Enter: Select/Deselect | C: Continue | Esc: Back")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(navigation, chunks[2]);
    }

    fn draw_gear_selection_static(f: &mut Frame, creation_state: &CharacterCreationState) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        // Title
        let title = Paragraph::new("⚔️ Gear Selection")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let gear_items: Vec<ListItem> = creation_state.available_gear_list.iter().enumerate().map(|(i, (gear_name, cost))| {
            let selected = creation_state.selected_gear.contains(gear_name);
            let is_current = i == creation_state.current_selection_index;
            let can_afford = creation_state.spent_gold + cost <= creation_state.starting_gold;
            
            let style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(Color::Green)
            } else if !can_afford {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            let prefix = if selected { "✓ " } else { "  " };
            ListItem::new(format!("{}{:<30} {} gp", prefix, gear_name, cost)).style(style)
        }).collect();

        let gear_list = List::new(gear_items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Available Gear (Gold: {}/{} | Spent: {})", 
                creation_state.starting_gold - creation_state.spent_gold,
                creation_state.starting_gold,
                creation_state.spent_gold
            )));
        f.render_widget(gear_list, chunks[1]);

        // Navigation
        let navigation = Paragraph::new("↑/↓: Navigate | Enter: Select/Deselect | C: Continue | Esc: Back")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(navigation, chunks[2]);
    }

    fn draw_character_confirmation_static(f: &mut Frame, creation_state: &CharacterCreationState) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(chunks[0]);

        // Title
        let title = Paragraph::new("Forge: Out of Chaos - Character Confirmation")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, left_chunks[0]);

        // Character summary
        let mut content = vec![
            Line::from(Span::styled("Step 4: Confirm Your Character", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
        ];

        if let (Some(rolled_data), Some(race), Some(name)) = (
            &creation_state.rolled_data,
            &creation_state.selected_race,
            &creation_state.character_name,
        ) {
            // Apply racial modifiers for display
            use crate::forge::ForgeCharacterCreation;
            let final_characteristics = ForgeCharacterCreation::apply_racial_modifiers(rolled_data, race);

            content.extend(vec![
                Line::from(Span::styled(format!("Name: {}", name), Style::default().fg(Color::Cyan))),
                Line::from(Span::styled(format!("Race: {}", race.name), Style::default().fg(Color::Cyan))),
                Line::from(""),
                Line::from(Span::styled("Final Characteristics:", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(format!("Strength:    {:.1}", final_characteristics.strength)),
                Line::from(format!("Stamina:     {:.1}", final_characteristics.stamina)),
                Line::from(format!("Intellect:   {:.1}", final_characteristics.intellect)),
                Line::from(format!("Insight:     {:.1}", final_characteristics.insight)),
                Line::from(format!("Dexterity:   {:.1}", final_characteristics.dexterity)),
                Line::from(format!("Awareness:   {:.1}", final_characteristics.awareness)),
                Line::from(format!("Speed:       {}", final_characteristics.speed)),
                Line::from(format!("Power:       {}", final_characteristics.power)),
                Line::from(format!("Luck:        {}", final_characteristics.luck)),
                Line::from(""),
                Line::from(Span::styled("Special Abilities:", Style::default().fg(Color::Green))),
            ]);

            for ability in &race.special_abilities {
                content.push(Line::from(format!("• {}", ability)));
            }

            content.extend(vec![
                Line::from(""),
                Line::from(Span::styled("Press ENTER to create character", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("Press ESC to go back and change name", Style::default().fg(Color::Yellow))),
            ]);
        } else {
            content.push(Line::from("Error: Missing character data"));
        }

        let confirmation = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Character Summary").border_style(Style::default().fg(Color::Green)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(confirmation, left_chunks[1]);

        // Race info panel
        if let Some(race) = &creation_state.selected_race {
            let race_info = vec![
                Line::from(Span::styled(&race.name, Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(race.description.as_str()),
                Line::from(""),
                Line::from(Span::styled("Starting Skills:", Style::default().fg(Color::Cyan))),
            ];

            let mut race_content = race_info;
            for (skill, level) in &race.starting_skills {
                race_content.push(Line::from(format!("• {} ({})", skill, level)));
            }

            let race_panel = Paragraph::new(race_content)
                .block(Block::default().borders(Borders::ALL).title("Race Details").border_style(Style::default().fg(Color::Cyan)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(race_panel, chunks[1]);
        }

        // Navigation
        let navigation = Paragraph::new("ENTER: Create Character | ESC: Go Back")
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Navigation").border_style(Style::default().fg(Color::Magenta)));
        f.render_widget(navigation, left_chunks[2]);
    }

    fn draw_character_list_static(f: &mut Frame, character_list: Option<&Vec<(String, chrono::DateTime<chrono::Utc>)>>, selected_index: Option<usize>) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("SAVED CHARACTERS")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, chunks[0]);

        // Character list content
        let content = if let Some(characters) = character_list {
            if characters.is_empty() {
                vec![
                    Line::from(""),
                    Line::from(Span::styled("No characters found", Style::default().fg(Color::DarkGray))),
                    Line::from(""),
                    Line::from("Create your first character by selecting"),
                    Line::from("'Create New Character' from the main menu."),
                    Line::from(""),
                    Line::from(Span::styled("Press any key to return to main menu", Style::default().fg(Color::Green))),
                ]
            } else {
                // Sort characters by last played (most recent first)
                let mut sorted_chars = characters.clone();
                sorted_chars.sort_by(|a, b| b.1.cmp(&a.1));

                let mut lines = vec![
                    Line::from(Span::styled("Your Saved Characters:".to_string(), Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                ];

                for (index, (name, last_played)) in sorted_chars.into_iter().enumerate() {
                    let time_str = last_played.format("%Y-%m-%d %H:%M UTC").to_string();
                    let is_selected = selected_index == Some(index);
                    let is_most_recent = index == 0;
                    
                    let (color, modifier, prefix) = if is_selected {
                        (Color::Black, Modifier::BOLD, "► ")
                    } else if is_most_recent {
                        (Color::Green, Modifier::BOLD, "  ")
                    } else {
                        (Color::White, Modifier::empty(), "  ")
                    };
                    
                    let index_str = format!("{}. ", index + 1);
                    let char_line = format!("{}{}{}", prefix, index_str, name);
                    let time_line = format!("     Last played: {}", time_str);
                    
                    let char_style = if is_selected {
                        Style::default().fg(color).bg(Color::Yellow).add_modifier(modifier)
                    } else {
                        Style::default().fg(color).add_modifier(modifier)
                    };
                    
                    lines.push(Line::from(Span::styled(char_line, char_style)));
                    lines.push(Line::from(time_line));
                    lines.push(Line::from(""));
                }

                lines.extend(vec![
                    Line::from(Span::styled("Navigation:".to_string(), Style::default().fg(Color::Cyan))),
                    Line::from("↑/↓ or W/S: Select character"),
                    Line::from("ENTER: Play selected character"),
                    Line::from("ESC: Return to main menu"),
                    Line::from(""),
                    Line::from(Span::styled("Select a character and press ENTER to play!".to_string(), Style::default().fg(Color::Green))),
                ]);

                lines
            }
        } else {
            vec![
                Line::from(""),
                Line::from(Span::styled("Loading character list...", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from("Please wait while we retrieve your characters."),
            ]
        };

        let character_list_widget = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Character Information").border_style(Style::default().fg(Color::Green)))
            .alignment(Alignment::Left);
        f.render_widget(character_list_widget, chunks[1]);

        // Instructions
        let instructions = if character_list.is_some() && !character_list.unwrap().is_empty() {
            Paragraph::new("↑/↓ W/S: Navigate | ENTER: Play Character | ESC: Main Menu | Q/Ctrl+C: Quit")
        } else {
            Paragraph::new("Any Key: Return to Main Menu | Q/Ctrl+C: Quit Game")
        };
        
        let instructions = instructions
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Controls").border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(instructions, chunks[2]);
    }

    fn draw_game_static(f: &mut Frame, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(chunks[0]);

        // Status bar - show actual character info if available
        let status_text = if let Some(character) = current_character {
            format!("{} | HP: {}/{} | STR: {:.1} | Level: {} | Gold: {}", 
                character.name,
                character.combat_stats.hit_points.current,
                character.combat_stats.hit_points.max,
                character.characteristics.strength,
                character.level,
                character.gold
            )
        } else {
            "No Character Loaded".to_string()
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Character Status").border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(status, left_chunks[0]);

        // Game world overview
        let world_content = if current_character.is_some() {
            vec![
                Line::from(Span::styled("🏰 WARLORDS REALM 🏰", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("Your journey from farm worker to mighty warlord begins!"),
                Line::from(""),
                Line::from(Span::styled("Available Actions:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("🗺️  Explore World - Venture into the unknown lands"),
                Line::from("   Discover new territories, find settlements, and encounter"),
                Line::from("   other travelers. Each zone holds unique challenges."),
                Line::from(""),
                Line::from("⚔️  Practice Combat - Test your skills in battle"),
                Line::from("   Fight wild creatures to gain experience and improve"),
                Line::from("   your combat abilities. Beware of stronger foes!"),
                Line::from(""),
                Line::from("📋 Character Menu - View detailed character information"),
                Line::from("   Check your skills, inventory, and character progression."),
                Line::from("   Access comprehensive character details and statistics."),
                Line::from(""),
                Line::from(Span::styled("World Status:", Style::default().fg(Color::Cyan))),
                Line::from("• World Generation: Ready"),
                Line::from("• Current Location: Central Lands"),
                Line::from("• Time of Day: Morning"),
                Line::from("• Weather: Clear"),
                Line::from(""),
                Line::from(Span::styled("Choose your path wisely, adventurer!", Style::default().fg(Color::Yellow))),
            ]
        } else {
            vec![
                Line::from("No character loaded."),
                Line::from("Please create or log in to a character first."),
            ]
        };

        let world = Paragraph::new(world_content)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Game World").border_style(Style::default().fg(Color::Green)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(world, left_chunks[1]);

        // Character details panel
        if let Some(character) = current_character {
            let mut character_info = vec![
                Line::from(Span::styled("Character Details", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(format!("Name: {}", character.name)),
                Line::from(format!("Race: {}", character.race.name)),
                Line::from(format!("Level: {}", character.level)),
                Line::from(format!("Experience: {}", character.experience)),
                Line::from(""),
                Line::from(Span::styled("Characteristics:", Style::default().fg(Color::Cyan))),
                Line::from(format!("STR: {:.1}", character.characteristics.strength)),
                Line::from(format!("STA: {:.1}", character.characteristics.stamina)),
                Line::from(format!("INT: {:.1}", character.characteristics.intellect)),
                Line::from(format!("INS: {:.1}", character.characteristics.insight)),
                Line::from(format!("DEX: {:.1}", character.characteristics.dexterity)),
                Line::from(format!("AWR: {:.1}", character.characteristics.awareness)),
                Line::from(format!("SPD: {}", character.characteristics.speed)),
                Line::from(format!("POW: {}", character.characteristics.power)),
                Line::from(format!("LUC: {}", character.characteristics.luck)),
                Line::from(""),
                Line::from(Span::styled("Combat Stats:", Style::default().fg(Color::Red))),
                Line::from(format!("Attack: {}", character.combat_stats.attack_value)),
                Line::from(format!("Defense: {}", character.combat_stats.defensive_value)),
                Line::from(format!("Damage: {:+}", character.combat_stats.damage_bonus)),
                Line::from(""),
                Line::from(Span::styled("Skills:", Style::default().fg(Color::Green))),
            ];

            for (skill, level) in &character.skills {
                character_info.push(Line::from(format!("• {} ({})", skill, level)));
            }

            let character_panel = Paragraph::new(character_info)
                .block(Block::default().borders(Borders::ALL).title("Character Sheet").border_style(Style::default().fg(Color::Magenta)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(character_panel, chunks[1]);
        } else {
            let no_char = Paragraph::new("No character loaded.\nCreate or log in to\na character to view\ndetailed information.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Character Sheet").border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(no_char, chunks[1]);
        }

        // Controls
        let controls = Paragraph::new("E: Explore World | F: Practice Combat | C: Character Menu | M: Main Menu | Q/Ctrl+C: Quit")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Controls").border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(controls, left_chunks[2]);
    }

    fn draw_character_menu_static(f: &mut Frame, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        if let Some(character) = current_character {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(chunks[0]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(15),
                    Constraint::Min(0),
                ])
                .split(chunks[1]);

            // Title
            let title = Paragraph::new(format!("Character Menu - {}", character.name))
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
            f.render_widget(title, left_chunks[0]);

            // Detailed character information
            let character_details = vec![
                Line::from(Span::styled("Character Information", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(format!("Name: {}", character.name)),
                Line::from(format!("Race: {}", character.race.name)),
                Line::from(format!("Level: {}", character.level)),
                Line::from(format!("Experience: {}", character.experience)),
                Line::from(format!("Gold: {}", character.gold)),
                Line::from(""),
                Line::from(Span::styled("Race Description:", Style::default().fg(Color::Cyan))),
                Line::from(character.race.description.as_str()),
                Line::from(""),
                Line::from(Span::styled("Special Abilities:", Style::default().fg(Color::Green))),
            ];

            let mut details = character_details;
            for ability in &character.race.special_abilities {
                details.push(Line::from(format!("• {}", ability)));
            }

            details.extend(vec![
                Line::from(""),
                Line::from(Span::styled("Inventory:", Style::default().fg(Color::Magenta))),
            ]);

            for item in &character.inventory {
                details.push(Line::from(format!("• {}", item)));
            }

            details.extend(vec![
                Line::from(""),
                Line::from(format!("Created: {}", character.created_at.format("%Y-%m-%d %H:%M"))),
                Line::from(format!("Last Played: {}", character.last_played.format("%Y-%m-%d %H:%M"))),
            ]);

            let character_info = Paragraph::new(details)
                .block(Block::default().borders(Borders::ALL).title("Character Details").border_style(Style::default().fg(Color::Green)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(character_info, left_chunks[1]);

            // Characteristics panel
            let characteristics = vec![
                Line::from(Span::styled("Characteristics", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(format!("Strength:    {:.1}", character.characteristics.strength)),
                Line::from(format!("Stamina:     {:.1}", character.characteristics.stamina)),
                Line::from(format!("Intellect:   {:.1}", character.characteristics.intellect)),
                Line::from(format!("Insight:     {:.1}", character.characteristics.insight)),
                Line::from(format!("Dexterity:   {:.1}", character.characteristics.dexterity)),
                Line::from(format!("Awareness:   {:.1}", character.characteristics.awareness)),
                Line::from(format!("Speed:       {}", character.characteristics.speed)),
                Line::from(format!("Power:       {}", character.characteristics.power)),
                Line::from(format!("Luck:        {}", character.characteristics.luck)),
            ];

            let char_panel = Paragraph::new(characteristics)
                .block(Block::default().borders(Borders::ALL).title("Characteristics").border_style(Style::default().fg(Color::Cyan)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(char_panel, right_chunks[0]);

            // Combat stats and skills
            let mut combat_skills = vec![
                Line::from(Span::styled("Combat Statistics", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(format!("Hit Points:    {}/{}", character.combat_stats.hit_points.current, character.combat_stats.hit_points.max)),
                Line::from(format!("Attack Value:  {}", character.combat_stats.attack_value)),
                Line::from(format!("Defense Value: {}", character.combat_stats.defensive_value)),
                Line::from(format!("Damage Bonus:  {:+}", character.combat_stats.damage_bonus)),
                Line::from(""),
                Line::from(Span::styled("Skills", Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];

            for (skill, level) in &character.skills {
                combat_skills.push(Line::from(format!("{}: {}", skill, level)));
            }

            let combat_panel = Paragraph::new(combat_skills)
                .block(Block::default().borders(Borders::ALL).title("Combat & Skills").border_style(Style::default().fg(Color::Red)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(combat_panel, right_chunks[1]);

            // Controls
            let controls = Paragraph::new("ESC/M: Return to Game | Q/Ctrl+C: Quit")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Controls").border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(controls, left_chunks[2]);
        } else {
            let no_char = Paragraph::new("No character loaded.")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Character Menu").border_style(Style::default().fg(Color::Red)));
            f.render_widget(no_char, area);
        }
    }

    fn draw_world_exploration_static(f: &mut Frame, world_state: &WorldExplorationState, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        // Main layout: 2/3 for world/status, 1/3 for messages
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(67),  // Top area for world and status (2/3)
                Constraint::Percentage(33),  // Bottom dialog area (1/3)
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main_chunks[0]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(0),      // World view
                Constraint::Length(3),   // Controls
            ])
            .split(top_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),  // Status panel
                Constraint::Percentage(40),  // Legend panel
            ])
            .split(top_chunks[1]);

        // Title with zone coordinates
        let title_text = format!("World Exploration - Zone ({}, {})", 
            world_state.current_zone.x, world_state.current_zone.y);
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(title, left_chunks[0]);

        // Generate world view from actual zone data - calculate available space
        let available_height = left_chunks[1].height.saturating_sub(3); // Subtract borders and title
        let available_width = left_chunks[1].width.saturating_sub(2); // Subtract borders
        let world_content = Self::generate_world_view(world_state, available_width as i32, available_height as i32);
        
        let world = Paragraph::new(world_content)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL).title("World View").border_style(Style::default().fg(Color::Green)));
        f.render_widget(world, left_chunks[1]);

        // Status panel
        let mut status_lines = vec![
            Line::from(Span::styled("Zone Information", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(format!("Zone: ({}, {})", world_state.current_zone.x, world_state.current_zone.y)),
            Line::from(format!("Position: ({}, {})", world_state.player_local_pos.x, world_state.player_local_pos.y)),
            Line::from(""),
        ];

        if let Some(zone_data) = &world_state.zone_data {
            let settlement_count = zone_data.settlements.len();
            let road_count = zone_data.roads.roads.len();
            
            status_lines.extend(vec![
                Line::from(Span::styled("Zone Contents:", Style::default().fg(Color::Cyan))),
                Line::from(format!("Settlements: {}", settlement_count)),
                Line::from(format!("Roads: {}", road_count)),
                Line::from(""),
            ]);

            // Show nearby settlements
            if !zone_data.settlements.is_empty() {
                status_lines.push(Line::from(Span::styled("Settlements:", Style::default().fg(Color::Green))));
                for settlement in &zone_data.settlements {
                    let distance = ((settlement.position.x - world_state.player_local_pos.x).pow(2) + 
                                  (settlement.position.y - world_state.player_local_pos.y).pow(2)) as f32;
                    let distance = (distance.sqrt()) as i32;
                    
                    let settlement_type = match settlement.settlement_type {
                        crate::world::SettlementType::Outpost => "Outpost",
                        crate::world::SettlementType::Village => "Village",
                        crate::world::SettlementType::Town => "Town", 
                        crate::world::SettlementType::City => "City",
                        crate::world::SettlementType::Capital => "Capital",
                    };
                    
                    status_lines.push(Line::from(format!("  {} {} ({}u away)", 
                        settlement_type, settlement.name, distance)));
                }
                status_lines.push(Line::from(""));
            }

            // Current terrain info
            let terrain_data = &zone_data.terrain;
            if let Some(tiles) = terrain_data.tiles.get(world_state.player_local_pos.y as usize) {
                if let Some(tile) = tiles.get(world_state.player_local_pos.x as usize) {
                    let terrain_name = match tile.terrain_type {
                        crate::world::TerrainType::Ocean => "Ocean",
                        crate::world::TerrainType::Lake => "Lake",
                        crate::world::TerrainType::River => "River",
                        crate::world::TerrainType::Plains => "Plains",
                        crate::world::TerrainType::Grassland => "Grassland",
                        crate::world::TerrainType::Forest => "Forest",
                        crate::world::TerrainType::Hill => "Hill",
                        crate::world::TerrainType::Mountain => "Mountain",
                        crate::world::TerrainType::Desert => "Desert",
                        crate::world::TerrainType::Swamp => "Swamp",
                        crate::world::TerrainType::Snow => "Snow",
                        crate::world::TerrainType::Tundra => "Tundra",
                    };
                    status_lines.extend(vec![
                        Line::from(Span::styled("Current Location:", Style::default().fg(Color::Yellow))),
                        Line::from(format!("Terrain: {}", terrain_name)),
                        Line::from(format!("Elevation: {}", tile.elevation)),
                    ]);
                }
            }
        } else {
            status_lines.push(Line::from(Span::styled("World data loading...", Style::default().fg(Color::DarkGray))));
        }

        if let Some(character) = current_character {
            status_lines.extend(vec![
                Line::from(""),
                Line::from(Span::styled("Character Status:", Style::default().fg(Color::Cyan))),
                Line::from(format!("HP: {}/{}", character.combat_stats.hit_points.current, character.combat_stats.hit_points.max)),
                Line::from(format!("Gold: {}", character.gold)),
            ]);
        }

        let status_panel = Paragraph::new(status_lines)
            .block(Block::default().borders(Borders::ALL).title("Status").border_style(Style::default().fg(Color::Cyan)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(status_panel, right_chunks[0]);

        // Legend panel
        let legend_lines = vec![
            Line::from(Span::styled("Legend:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("@ = You"),
            Line::from("█●○◦· = Settlements"),
            Line::from("MGTHR! = NPCs"),
            Line::from("⌂◊♜♠♦ = POIs"),
            Line::from("♣^▲.,~ = Terrain"),
            Line::from("═ = Roads"),
        ];
        let legend_panel = Paragraph::new(legend_lines)
            .block(Block::default().borders(Borders::ALL).title("Legend").border_style(Style::default().fg(Color::Yellow)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(legend_panel, right_chunks[1]);

        // Dialog area at bottom - show more messages since we have 1/3 of the screen
        let dialog_text = if world_state.messages.is_empty() {
            "Welcome to the world! Press L to look around, H for help, or start exploring with WASD.".to_string()
        } else {
            // Show more messages since we have a larger area (1/3 of screen)
            // Calculate approximate lines available: 1/3 of screen height minus borders
            let available_height = (f.size().height / 3).saturating_sub(2) as usize;
            let max_messages = available_height.max(8); // Show at least 8 messages
            
            world_state.messages.iter()
                .rev()
                .take(max_messages)
                .rev()
                .cloned()
                .collect::<Vec<String>>()
                .join("\n")
        };
        
        let dialog_panel = Paragraph::new(dialog_text)
            .block(Block::default().borders(Borders::ALL).title("Messages").border_style(Style::default().fg(Color::Green)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(dialog_panel, main_chunks[1]);
        
        // Controls
        let controls_text = vec![
            Line::from("WASD/Arrow Keys: Move | M: Menu | F: Fight | Q: Quit | H: Help"),
            Line::from("L: Look | E: Enter/Examine | P: POIs | T: Talk | R: Search | I: Interact | C: Camp | G: Gather"),
        ];
        let controls = Paragraph::new(controls_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Controls").border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(controls, left_chunks[2]);
    }

    fn generate_world_view(world_state: &WorldExplorationState, view_width: i32, view_height: i32) -> Vec<Line<'static>> {
        let mut world_content = vec![];
        
        if let Some(zone_data) = &world_state.zone_data {
            // Use available space for dynamic viewport sizing
            let actual_view_height = view_height.max(10);
            let actual_view_width = view_width.max(20);
            
            let half_width = actual_view_width / 2;
            let half_height = actual_view_height / 2;
            
            // Calculate world bounds - always center the player
            let start_x = world_state.player_local_pos.x - half_width;
            let end_x = world_state.player_local_pos.x + half_width;
            let start_y = world_state.player_local_pos.y - half_height;
            let end_y = world_state.player_local_pos.y + half_height;
            
            for y in start_y..=end_y {
                let mut line_spans = Vec::new();
                
                for x in start_x..=end_x {
                    // Always put player at center of viewport
                    let screen_x = x - start_x;
                    let screen_y = y - start_y;
                    let center_x = half_width;
                    let center_y = half_height;
                    
                    if screen_x == center_x && screen_y == center_y {
                        // Player always at center - bright yellow
                        line_spans.push(Span::styled("@", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    } else {
                        // Handle coordinates that might be outside current zone
                        let (zone_coord, local_x, local_y) = if x < 0 || x >= crate::world::ZONE_SIZE || y < 0 || y >= crate::world::ZONE_SIZE {
                            // Calculate which zone this coordinate belongs to
                            let zone_offset_x = if x < 0 { -1 } else if x >= crate::world::ZONE_SIZE { 1 } else { 0 };
                            let zone_offset_y = if y < 0 { -1 } else if y >= crate::world::ZONE_SIZE { 1 } else { 0 };
                            
                            let adjacent_zone = crate::world::ZoneCoord::new(
                                world_state.current_zone.x + zone_offset_x,
                                world_state.current_zone.y + zone_offset_y
                            );
                            
                            let local_x = if x < 0 { 
                                crate::world::ZONE_SIZE + x 
                            } else if x >= crate::world::ZONE_SIZE { 
                                x - crate::world::ZONE_SIZE 
                            } else { 
                                x 
                            };
                            
                            let local_y = if y < 0 { 
                                crate::world::ZONE_SIZE + y 
                            } else if y >= crate::world::ZONE_SIZE { 
                                y - crate::world::ZONE_SIZE 
                            } else { 
                                y 
                            };
                            
                            (Some(adjacent_zone), local_x, local_y)
                        } else {
                            (None, x, y)
                        };
                        
                        // For now, show void for adjacent zones (we'd need to load them for seamless transitions)
                        if zone_coord.is_some() {
                            line_spans.push(Span::styled("·", Style::default().fg(Color::DarkGray))); // Show faded terrain for adjacent zones
                            continue;
                        }
                        // Use the calculated local coordinates for lookups
                        let lookup_x = local_x;
                        let lookup_y = local_y;
                        
                        // Check for settlements first
                        let mut found_settlement = false;
                        for settlement in &zone_data.settlements {
                            if settlement.position.x == lookup_x && settlement.position.y == lookup_y {
                                found_settlement = true;
                                match settlement.settlement_type {
                                    crate::world::SettlementType::Capital => line_spans.push(Span::styled("█", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
                                    crate::world::SettlementType::City => line_spans.push(Span::styled("●", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                                    crate::world::SettlementType::Town => line_spans.push(Span::styled("○", Style::default().fg(Color::White))),
                                    crate::world::SettlementType::Village => line_spans.push(Span::styled("◦", Style::default().fg(Color::LightYellow))),
                                    crate::world::SettlementType::Outpost => line_spans.push(Span::styled("·", Style::default().fg(Color::Gray))),
                                }
                                break;
                            }
                        }
                        
                        if !found_settlement {
                            // Check for NPCs first
                            let mut found_npc = false;
                            for npc in &zone_data.npcs {
                                if npc.position.x == lookup_x && npc.position.y == lookup_y {
                                    found_npc = true;
                                    let npc_color = match npc.npc_type {
                                        crate::world::NPCType::Merchant => Color::Yellow,
                                        crate::world::NPCType::Guard => Color::Blue,
                                        crate::world::NPCType::Traveler => Color::Green,
                                        crate::world::NPCType::Hermit => Color::Gray,
                                        crate::world::NPCType::Scholar => Color::Cyan,
                                        crate::world::NPCType::Warrior => Color::Red,
                                        crate::world::NPCType::Thief => Color::DarkGray,
                                        crate::world::NPCType::Farmer => Color::LightGreen,
                                        crate::world::NPCType::Noble => Color::Magenta,
                                        crate::world::NPCType::Blacksmith => Color::Gray,
                                        crate::world::NPCType::Innkeeper => Color::LightYellow,
                                        crate::world::NPCType::Priest => Color::White,
                                        crate::world::NPCType::Ranger => Color::Green,
                                        crate::world::NPCType::Bandit => Color::Red,
                                        crate::world::NPCType::Explorer => Color::Cyan,
                                    };
                                    line_spans.push(Span::styled(npc.npc_type.get_ascii_char().to_string(), Style::default().fg(npc_color)));
                                    break;
                                }
                            }
                            
                            if !found_npc {
                                // Check for POIs (Points of Interest)
                                let mut found_poi = false;
                                for poi in &zone_data.points_of_interest {
                                    if poi.position.x == lookup_x && poi.position.y == lookup_y {
                                        found_poi = true;
                                        let (symbol, color) = match poi.poi_type {
                                            crate::world::PoiType::AncientRuins => ('⌂', Color::LightYellow),
                                            crate::world::PoiType::Cave => ('◊', Color::Gray),
                                            crate::world::PoiType::AbandonedTower => ('♜', Color::DarkGray),
                                            crate::world::PoiType::MysticShrine => ('♠', Color::Magenta),
                                            crate::world::PoiType::DragonLair => ('♦', Color::Red),
                                            crate::world::PoiType::BanditCamp => ('▲', Color::Red),
                                            crate::world::PoiType::WizardTower => ('♨', Color::Blue),
                                            crate::world::PoiType::Temple => ('⌘', Color::White),
                                            crate::world::PoiType::Crypt => ('◘', Color::DarkGray),
                                            crate::world::PoiType::TreasureVault => ('♛', Color::Yellow),
                                            _ => ('?', Color::White),
                                        };
                                        line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                                        break;
                                    }
                                }
                                
                                if !found_poi {
                                    // Check for roads
                                    let mut found_road = false;
                                    for road in &zone_data.roads.roads {
                                        for point in &road.path {
                                            if point.x == lookup_x && point.y == lookup_y {
                                                found_road = true;
                                                let road_color = match road.road_type {
                                                    crate::world::RoadType::Trail => Color::DarkGray,
                                                    crate::world::RoadType::Path => Color::Gray,
                                                    crate::world::RoadType::Road => Color::LightYellow,
                                                    crate::world::RoadType::Highway => Color::Yellow,
                                                    crate::world::RoadType::Imperial => Color::White,
                                                };
                                                line_spans.push(Span::styled("═", Style::default().fg(road_color)));
                                                break;
                                            }
                                        }
                                        if found_road { break; }
                                    }
                                
                                    if !found_road {
                                        // Show terrain with subtle colors
                                        if let Some(row) = zone_data.terrain.tiles.get(lookup_y as usize) {
                                            if let Some(tile) = row.get(lookup_x as usize) {
                                                let (symbol, base_color) = match tile.terrain_type {
                                                    crate::world::TerrainType::Ocean => ('~', Color::Blue),
                                                    crate::world::TerrainType::Lake => ('~', Color::Cyan),
                                                    crate::world::TerrainType::River => ('~', Color::LightBlue),
                                                    crate::world::TerrainType::Plains => ('.', Color::Yellow),
                                                    crate::world::TerrainType::Grassland => (',', Color::LightGreen),
                                                    crate::world::TerrainType::Forest => ('♣', Color::Green),
                                                    crate::world::TerrainType::Hill => ('^', Color::LightGreen),
                                                    crate::world::TerrainType::Mountain => ('▲', Color::White),
                                                    crate::world::TerrainType::Desert => ('·', Color::LightYellow),
                                                    crate::world::TerrainType::Swamp => ('≈', Color::DarkGray),
                                                    crate::world::TerrainType::Snow => ('*', Color::White),
                                                    crate::world::TerrainType::Tundra => (':', Color::Gray),
                                                };
                                                
                                                // Add subtle variation based on elevation and fertility
                                                let mut style = Style::default().fg(base_color);
                                                
                                                // Higher elevation areas are brighter
                                                if tile.elevation > 75.0 {
                                                    style = style.add_modifier(Modifier::BOLD);
                                                } else if tile.elevation < 25.0 {
                                                    // Lower elevation areas are slightly darker
                                                    style = match base_color {
                                                        Color::Green => style.fg(Color::DarkGray),
                                                        Color::LightGreen => style.fg(Color::Green),
                                                        Color::Yellow => style.fg(Color::DarkGray),
                                                        Color::LightYellow => style.fg(Color::Yellow),
                                                        _ => style,
                                                    };
                                                }
                                                
                                                // Very fertile areas have enhanced green colors (except water/snow/desert)
                                                if tile.fertility > 0.8 && matches!(tile.terrain_type, 
                                                    crate::world::TerrainType::Plains | 
                                                    crate::world::TerrainType::Grassland |
                                                    crate::world::TerrainType::Hill |
                                                    crate::world::TerrainType::Forest) {
                                                    style = style.fg(Color::LightGreen);
                                                } else if tile.fertility < 0.3 && !matches!(tile.terrain_type,
                                                    crate::world::TerrainType::Ocean | 
                                                    crate::world::TerrainType::Lake | 
                                                    crate::world::TerrainType::River |
                                                    crate::world::TerrainType::Desert |
                                                    crate::world::TerrainType::Snow) {
                                                    // Poor fertility areas are more brown/gray
                                                    style = style.fg(Color::DarkGray);
                                                }
                                                
                                                line_spans.push(Span::styled(symbol.to_string(), style));
                                            } else {
                                                line_spans.push(Span::styled("?", Style::default().fg(Color::Red)));
                                            }
                                        } else {
                                            line_spans.push(Span::styled("?", Style::default().fg(Color::Red)));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                world_content.push(Line::from(line_spans));
            }
        } else {
            world_content = vec![
                Line::from("Generating world..."),
                Line::from(""),
                Line::from("Please wait while the world data loads."),
            ];
        }
        
        world_content
    }

    fn draw_dungeon_exploration_static(f: &mut Frame, dungeon_state: &DungeonExplorationState, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        // Main layout: 2/3 for dungeon view/status, 1/3 for messages
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(67),  // Top area for dungeon and status (2/3)
                Constraint::Percentage(33),  // Bottom dialog area (1/3)
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main_chunks[0]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(0),      // Dungeon view
                Constraint::Length(3),   // Controls
            ])
            .split(top_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),  // Status panel
                Constraint::Percentage(50),  // Floor info panel
            ])
            .split(top_chunks[1]);

        // Title with dungeon name and floor
        let title_text = format!("{} - Floor {}", 
            dungeon_state.dungeon.name, 
            dungeon_state.dungeon.current_floor + 1);
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));
        f.render_widget(title, left_chunks[0]);

        // Generate dungeon view
        let available_height = left_chunks[1].height.saturating_sub(2); // Subtract borders
        let available_width = left_chunks[1].width.saturating_sub(2); // Subtract borders
        let dungeon_content = Self::generate_dungeon_view(dungeon_state, available_width as i32, available_height as i32);
        
        let dungeon = Paragraph::new(dungeon_content)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)));
        f.render_widget(dungeon, left_chunks[1]);

        // Controls at bottom
        let controls = Paragraph::new("WASD/Arrows: Move | (E)xamine | (I)nteract | (F)ight | (U)se stairs | (L)ook | (X)it dungeon | Ctrl+Q: Quit")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
        f.render_widget(controls, left_chunks[2]);

        // Character status (right top)
        let status_content = if let Some(character) = current_character {
            vec![
                Line::from(format!("Character: {}", character.name)),
                Line::from(format!("Level: {} ({})", character.level, character.race.name)),
                Line::from(format!("HP: {}/{}", character.combat_stats.hit_points.current, character.combat_stats.hit_points.max)),
                Line::from(format!("Position: ({}, {})", dungeon_state.player_pos.x, dungeon_state.player_pos.y)),
                Line::from(format!("Turn: {}", dungeon_state.turn_count)),
                Line::from(""),
                Line::from("Equipment:"),
                Line::from("• Simple tools"),
                Line::from("• Farm clothes"),
            ]
        } else {
            vec![Line::from("No character loaded")]
        };

        let status = Paragraph::new(status_content)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().title("Status").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(status, right_chunks[0]);

        // Floor info (right bottom)
        let floor_info = if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            vec![
                Line::from(format!("Floor {}", dungeon_state.dungeon.current_floor + 1)),
                Line::from(format!("Rooms: {}", floor.rooms.len())),
                Line::from(format!("Creatures: {}", floor.creatures.len())),
                Line::from(format!("Features: {}", floor.features.len())),
                Line::from(""),
                Line::from("Visible Creatures:"),
            ]
        } else {
            vec![Line::from("Floor data not available")]
        };

        let floor_panel = Paragraph::new(floor_info)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().title("Floor Info").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
        f.render_widget(floor_panel, right_chunks[1]);

        // Messages area (bottom)
        let message_content: Vec<Line> = dungeon_state.messages.iter()
            .rev()
            .take(10)
            .rev()
            .map(|msg| Line::from(msg.clone()))
            .collect();

        let messages = Paragraph::new(message_content)
            .style(Style::default().fg(Color::White))
            .block(Block::default().title("Messages").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(messages, main_chunks[1]);
    }

    fn generate_dungeon_view(dungeon_state: &DungeonExplorationState, view_width: i32, view_height: i32) -> Vec<Line<'static>> {
        let mut dungeon_content = Vec::new();
        
        if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            let player_x = dungeon_state.player_pos.x;
            let player_y = dungeon_state.player_pos.y;
            
            // Calculate viewport bounds centered on player
            let half_width = view_width / 2;
            let half_height = view_height / 2;
            let start_x = (player_x - half_width).max(0);
            let end_x = (player_x + half_width).min(crate::world::DUNGEON_WIDTH - 1);
            let start_y = (player_y - half_height).max(0);
            let end_y = (player_y + half_height).min(crate::world::DUNGEON_HEIGHT - 1);
            
            for y in start_y..=end_y {
                let mut line_spans = Vec::new();
                
                for x in start_x..=end_x {
                    if x == player_x && y == player_y {
                        // Player position
                        line_spans.push(Span::styled("@", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)));
                    } else if let Some(creature) = floor.creatures.iter().find(|c| c.position.x == x && c.position.y == y) {
                        // Creature position - only show if tile is visible
                        if let Some(tile) = floor.tiles.get(y as usize).and_then(|row| row.get(x as usize)) {
                            if tile.visible {
                                let (symbol, color) = match creature.creature_type {
                                    crate::world::CreatureType::Skeleton => ('S', Color::White),
                                    crate::world::CreatureType::Zombie => ('Z', Color::Green),
                                    crate::world::CreatureType::Ghost => ('G', Color::Cyan),
                                    crate::world::CreatureType::Rat => ('r', Color::Red),
                                    crate::world::CreatureType::Bat => ('b', Color::Gray),
                                    crate::world::CreatureType::Spider => ('s', Color::Red),
                                    crate::world::CreatureType::Goblin => ('g', Color::LightGreen),
                                    crate::world::CreatureType::Orc => ('O', Color::Red),
                                    crate::world::CreatureType::Bandit => ('B', Color::Red),
                                    crate::world::CreatureType::GuardianSpirit => ('*', Color::LightBlue),
                                    crate::world::CreatureType::WildAnimal => ('a', Color::Yellow),
                                    crate::world::CreatureType::Construct => ('C', Color::Gray),
                                };
                                line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                            } else {
                                // Creature not visible - fall through to tile rendering
                                if tile.explored {
                                    let (symbol, color) = match &tile.tile_type {
                                        crate::world::DungeonTileType::Wall => ('#', Color::DarkGray),
                                        crate::world::DungeonTileType::Floor => ('.', Color::Gray),
                                        crate::world::DungeonTileType::Door(state) => {
                                            match state {
                                                crate::world::DoorState::Open => ('+', Color::Gray),
                                                crate::world::DoorState::Closed => ('D', Color::Gray),
                                                crate::world::DoorState::Locked => ('L', Color::Gray),
                                                crate::world::DoorState::Secret => ('#', Color::Gray), // Secret doors look like walls when not visible
                                            }
                                        }
                                        crate::world::DungeonTileType::Stairs(_) => ('<', Color::Gray),
                                        crate::world::DungeonTileType::Chest => ('$', Color::Gray),
                                        crate::world::DungeonTileType::Altar => ('A', Color::Gray),
                                        crate::world::DungeonTileType::Water => ('~', Color::Gray),
                                        crate::world::DungeonTileType::Pit => ('O', Color::Gray),
                                        crate::world::DungeonTileType::Rubble => ('&', Color::Gray),
                                        crate::world::DungeonTileType::Pillar => ('|', Color::Gray),
                                        crate::world::DungeonTileType::Window => ('=', Color::Gray),
                                        crate::world::DungeonTileType::Torch => ('*', Color::Gray),
                                    };
                                    line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                                } else {
                                    line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                                }
                            }
                        } else {
                            // No tile data - render as empty
                            line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                        }
                    } else if let Some(loot_pile) = floor.loot_piles.iter().find(|lp| lp.position.x == x && lp.position.y == y) {
                        // Loot pile - show if tile is visible
                        if let Some(tile) = floor.tiles.get(y as usize).and_then(|row| row.get(x as usize)) {
                            if tile.visible {
                                let symbol = if loot_pile.discovered { '$' } else { '?' };
                                line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)));
                            } else {
                                // Fall through to normal tile rendering
                                if tile.explored {
                                    let (symbol, color) = match &tile.tile_type {
                                        crate::world::DungeonTileType::Wall => ('#', Color::DarkGray),
                                        crate::world::DungeonTileType::Floor => ('.', Color::Gray),
                                        _ => ('.', Color::Gray),
                                    };
                                    line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                                } else {
                                    line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                                }
                            }
                        } else {
                            line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                        }
                    } else if let Some(corpse) = floor.corpses.iter().find(|c| c.position.x == x && c.position.y == y) {
                        // Corpse - show if tile is visible
                        if let Some(tile) = floor.tiles.get(y as usize).and_then(|row| row.get(x as usize)) {
                            if tile.visible {
                                let (symbol, color) = match corpse.decay_level {
                                    0..=2 => ('%', Color::Red),        // Fresh corpse - red
                                    3..=6 => ('%', Color::Yellow),     // Decaying corpse - yellow
                                    7..=9 => ('%', Color::White),      // Old corpse - white
                                    _ => ('☠', Color::Gray),           // Skeleton remains - gray
                                };
                                line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                            } else {
                                // Fall through to normal tile rendering
                                if tile.explored {
                                    let (symbol, color) = match &tile.tile_type {
                                        crate::world::DungeonTileType::Wall => ('#', Color::DarkGray),
                                        crate::world::DungeonTileType::Floor => ('.', Color::Gray),
                                        _ => ('.', Color::Gray),
                                    };
                                    line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                                } else {
                                    line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                                }
                            }
                        } else {
                            line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                        }
                    } else if let Some(tile) = floor.tiles.get(y as usize).and_then(|row| row.get(x as usize)) {
                        // Tile rendering
                        if tile.visible || tile.explored {
                            let (symbol, color) = match &tile.tile_type {
                                crate::world::DungeonTileType::Wall => ('#', Color::Gray),
                                crate::world::DungeonTileType::Floor => ('.', Color::White),
                                crate::world::DungeonTileType::Door(state) => {
                                    match state {
                                        crate::world::DoorState::Open => ('+', Color::Yellow),
                                        crate::world::DoorState::Closed => ('|', Color::Yellow),
                                        crate::world::DoorState::Locked => ('X', Color::Red),
                                        crate::world::DoorState::Secret => ('#', Color::Gray), // Hidden
                                    }
                                },
                                crate::world::DungeonTileType::Stairs(stair_type) => {
                                    match stair_type {
                                        crate::world::StairType::Up => ('<', Color::LightBlue),
                                        crate::world::StairType::Down => ('>', Color::LightBlue),
                                        crate::world::StairType::UpDown => ('=', Color::LightBlue),
                                    }
                                },
                                crate::world::DungeonTileType::Water => ('~', Color::Blue),
                                crate::world::DungeonTileType::Pit => ('O', Color::Red),
                                crate::world::DungeonTileType::Rubble => ('*', Color::Gray),
                                crate::world::DungeonTileType::Altar => ('A', Color::LightMagenta),
                                crate::world::DungeonTileType::Chest => ('C', Color::Yellow),
                                crate::world::DungeonTileType::Pillar => ('I', Color::White),
                                crate::world::DungeonTileType::Window => ('W', Color::LightBlue),
                                crate::world::DungeonTileType::Torch => ('T', Color::LightRed),
                            };
                            
                            // Adjust brightness based on light level and visibility
                            let adjusted_color = if tile.visible {
                                color
                            } else {
                                // Dimmed for explored but not currently visible
                                match color {
                                    Color::White => Color::Gray,
                                    Color::LightBlue => Color::Blue,
                                    Color::LightYellow => Color::Yellow,
                                    Color::LightGreen => Color::Green,
                                    Color::LightRed => Color::Red,
                                    Color::LightMagenta => Color::Magenta,
                                    c => c,
                                }
                            };
                            
                            line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(adjusted_color)));
                        } else {
                            // Unexplored area
                            line_spans.push(Span::styled(" ".to_string(), Style::default().fg(Color::Black)));
                        }
                    } else {
                        // Out of bounds
                        line_spans.push(Span::styled(" ".to_string(), Style::default().fg(Color::Black)));
                    }
                }
                
                dungeon_content.push(Line::from(line_spans));
            }
        } else {
            dungeon_content = vec![
                Line::from("Loading dungeon..."),
                Line::from(""),
                Line::from("Please wait while the dungeon data loads."),
            ];
        }
        
        dungeon_content
    }

    fn draw_combat_static(f: &mut Frame, combat_state: &CombatState) {
        let area = f.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Min(8),
                Constraint::Length(6),
                Constraint::Length(3),
            ])
            .split(area);

        // Combat title
        let title = Paragraph::new(format!("⚔️  COMBAT - Round {} ⚔️", combat_state.encounter.round))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)));
        f.render_widget(title, chunks[0]);

        // Combatants status
        let mut status_lines = vec![Line::from(Span::styled("Combatants:", Style::default().add_modifier(Modifier::BOLD)))];
        for (i, participant) in combat_state.encounter.participants.iter().enumerate() {
            let hp_ratio = participant.combat_stats.hit_points.current as f32 / participant.combat_stats.hit_points.max as f32;
            let hp_color = if hp_ratio > 0.5 { Color::Green } else if hp_ratio > 0.25 { Color::Yellow } else { Color::Red };
            
            let is_current = i == combat_state.encounter.current_turn;
            let turn_indicator = if is_current { "► " } else { "  " };
            
            let armor_info = if let Some(armor) = &participant.armor {
                format!(" | Armor: {}/{} (AR: {})", 
                    armor.armor_points, 
                    armor.max_armor_points,
                    armor.get_current_armor_rating())
            } else {
                String::new()
            };
            
            let line = format!("{}{} - HP: {}/{} | AV: {} | DV: {}{}",
                turn_indicator,
                participant.name,
                participant.combat_stats.hit_points.current,
                participant.combat_stats.hit_points.max,
                participant.get_total_attack_value(),
                participant.get_total_defense_value(),
                armor_info
            );
            
            let style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if !participant.is_alive() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(hp_color)
            };
            
            status_lines.push(Line::from(Span::styled(line, style)));
        }
        
        let status = Paragraph::new(status_lines)
            .block(Block::default().borders(Borders::ALL).title("Status").border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(status, chunks[1]);

        // Combat log
        let log_start = combat_state.encounter.combat_log.len().saturating_sub(10);
        let recent_logs: Vec<Line> = combat_state.encounter.combat_log[log_start..]
            .iter()
            .map(|log| Line::from(log.as_str()))
            .collect();
        
        let combat_log = Paragraph::new(recent_logs)
            .block(Block::default().borders(Borders::ALL).title("Combat Log").border_style(Style::default().fg(Color::White)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(combat_log, chunks[2]);

        // Actions based on combat phase
        if let Some(current) = combat_state.encounter.get_current_participant() {
            if current.is_player && current.is_alive() {
                match combat_state.combat_phase {
                    CombatPhase::InitiativeRoll => {
                        let init_items = vec![
                            ListItem::new("Combat is about to begin!").style(Style::default().fg(Color::Yellow)),
                            ListItem::new("Initiative will be rolled for all participants").style(Style::default().fg(Color::White)),
                            ListItem::new("Press ENTER to roll initiative").style(Style::default().fg(Color::Green)),
                        ];
                        
                        let actions = List::new(init_items)
                            .block(Block::default().borders(Borders::ALL)
                                .title("Rolling Initiative")
                                .border_style(Style::default().fg(Color::Yellow)));
                        f.render_widget(actions, chunks[3]);
                    }
                    CombatPhase::DeclaringActions => {
                        let declare_items = vec![
                            ListItem::new("All participants declare their actions").style(Style::default().fg(Color::Yellow)),
                            ListItem::new("Actions will be resolved in initiative order").style(Style::default().fg(Color::White)),
                            ListItem::new("Press ENTER to continue").style(Style::default().fg(Color::Green)),
                        ];
                        
                        let actions = List::new(declare_items)
                            .block(Block::default().borders(Borders::ALL)
                                .title("Declaring Actions")
                                .border_style(Style::default().fg(Color::Blue)));
                        f.render_widget(actions, chunks[3]);
                    }
                    CombatPhase::SelectingSkill => {
                        // Calculate visible range for scrolling
                        let max_visible = 5; // Show 5 skills at a time
                        let total_skills = combat_state.available_skills.len();
                        
                        // Adjust offset if needed
                        let offset = if combat_state.current_skill_index >= combat_state.skill_list_offset + max_visible {
                            combat_state.current_skill_index - max_visible + 1
                        } else if combat_state.current_skill_index < combat_state.skill_list_offset {
                            combat_state.current_skill_index
                        } else {
                            combat_state.skill_list_offset
                        };
                        
                        let visible_end = (offset + max_visible).min(total_skills);
                        
                        let mut skill_items = Vec::new();
                        for i in offset..visible_end {
                            let skill = &combat_state.available_skills[i];
                            let is_selected = i == combat_state.current_skill_index;
                            
                            let style = if is_selected {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            };
                            
                            let prefix = if is_selected { "► " } else { "  " };
                            skill_items.push(ListItem::new(format!("{}{}", prefix, skill)).style(style));
                        }
                        
                        // Add scroll indicators
                        let scroll_info = if total_skills > max_visible {
                            format!(" ({}/{}) ▼▲ to scroll", 
                                combat_state.current_skill_index + 1, 
                                total_skills)
                        } else {
                            String::new()
                        };
                        
                        let actions = List::new(skill_items)
                            .block(Block::default().borders(Borders::ALL)
                                .title(format!("{}'s Turn - Select Skill/Spell/Action{}", current.name, scroll_info))
                                .border_style(Style::default().fg(Color::Green)));
                        f.render_widget(actions, chunks[3]);
                    }
                    CombatPhase::SelectingTarget => {
                        let mut target_items = Vec::new();
                        let mut enemy_counter = 1;
                        
                        for participant in &combat_state.encounter.participants {
                            if !participant.is_player && participant.is_alive() {
                                let target_text = format!("{}. {} (HP: {}/{})", 
                                    enemy_counter, 
                                    participant.name,
                                    participant.combat_stats.hit_points.current,
                                    participant.combat_stats.hit_points.max);
                                target_items.push(ListItem::new(target_text).style(Style::default().fg(Color::Red)));
                                enemy_counter += 1;
                            }
                        }
                        
                        let default_skill = "Unknown".to_string();
                        let skill_name = combat_state.selected_skill.as_ref().unwrap_or(&default_skill);
                        let actions = List::new(target_items)
                            .block(Block::default().borders(Borders::ALL)
                                .title(format!("Using {} - Select Target", skill_name))
                                .border_style(Style::default().fg(Color::Red)));
                        f.render_widget(actions, chunks[3]);
                    }
                    CombatPhase::ResolvingActions => {
                        let resolving = Paragraph::new("Resolving actions...")
                            .style(Style::default().fg(Color::Yellow))
                            .alignment(Alignment::Center)
                            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
                        f.render_widget(resolving, chunks[3]);
                    }
                    CombatPhase::RoundComplete => {
                        let round_items = vec![
                            ListItem::new("Round completed!").style(Style::default().fg(Color::Cyan)),
                            ListItem::new("Preparing for next round").style(Style::default().fg(Color::White)),
                            ListItem::new("Press ENTER to continue").style(Style::default().fg(Color::Green)),
                        ];
                        
                        let actions = List::new(round_items)
                            .block(Block::default().borders(Borders::ALL)
                                .title("Round Complete")
                                .border_style(Style::default().fg(Color::Cyan)));
                        f.render_widget(actions, chunks[3]);
                    }
                    CombatPhase::CombatComplete(_) => {
                        let complete = Paragraph::new("Combat Complete! Press ENTER to continue.")
                            .style(Style::default().fg(Color::Green))
                            .alignment(Alignment::Center)
                            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
                        f.render_widget(complete, chunks[3]);
                    }
                }
            } else {
                let waiting = Paragraph::new("Waiting for enemy turn...")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
                f.render_widget(waiting, chunks[3]);
            }
        }

        // Controls
        let controls = if combat_state.encounter.is_combat_over() {
            Paragraph::new("Combat Over! Press ENTER to continue | Q/Ctrl+C: Quit")
                .style(Style::default().fg(Color::Green))
        } else {
            match combat_state.combat_phase {
                CombatPhase::InitiativeRoll => {
                    Paragraph::new("ENTER: Roll Initiative | Q/Ctrl+C: Quit")
                        .style(Style::default().fg(Color::Yellow))
                }
                CombatPhase::DeclaringActions => {
                    Paragraph::new("ENTER: Declare Actions | Q/Ctrl+C: Quit")
                        .style(Style::default().fg(Color::Blue))
                }
                CombatPhase::SelectingSkill => {
                    Paragraph::new("↑/↓: Navigate | ENTER: Select | ESC: Cancel | Q/Ctrl+C: Quit")
                        .style(Style::default().fg(Color::Green))
                }
                CombatPhase::SelectingTarget => {
                    Paragraph::new("1-9: Select Target | ESC: Go Back | Q/Ctrl+C: Quit")
                        .style(Style::default().fg(Color::Red))
                }
                CombatPhase::ResolvingActions => {
                    Paragraph::new("Resolving all declared actions...")
                        .style(Style::default().fg(Color::Yellow))
                }
                CombatPhase::RoundComplete => {
                    Paragraph::new("Round complete! ENTER: Start next round")
                        .style(Style::default().fg(Color::Cyan))
                }
                CombatPhase::CombatComplete(_) => {
                    Paragraph::new("Combat Over! Press ENTER to continue")
                        .style(Style::default().fg(Color::Green))
                }
            }
        };
        
        let controls = controls
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(controls, chunks[4]);
    }

    pub fn handle_input(&self) -> anyhow::Result<Option<KeyEvent>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle Ctrl+C for graceful shutdown
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        return Ok(Some(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)));
                    }
                    return Ok(Some(key));
                }
                _ => {}
            }
        }
        Ok(None)
    }
}