use crossterm::{
    event::{self, Event, KeyEvent, KeyCode, KeyModifiers, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use crate::forge::{RolledCharacteristics, ForgeRace};
use rand::Rng;

pub mod framework;
pub mod layout;
pub mod components;
pub mod mouse;

pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

pub struct GameUI {
    terminal: TerminalType,
}

#[derive(Debug, Clone)]
pub enum UIState {
    Welcome,
    MainMenu,
    CharacterCreation(CharacterCreationState),
    CharacterList(Vec<(String, chrono::DateTime<chrono::Utc>)>, Option<usize>), // characters, selected_index
    Playing,
    CharacterMenu,
    CharacterSheet,
    InventoryManagement(InventoryState),
    EquipmentManagement(EquipmentState),
    WorldExploration(WorldExplorationState),
    DungeonExploration(DungeonExplorationState),
    Combat(CombatState),
    TacticalCombat(TacticalCombatState),
}

#[derive(Debug, Clone)]
pub struct WorldExplorationState {
    pub current_zone: crate::world::ZoneCoord,
    pub player_local_pos: crate::world::LocalCoord,
    pub zone_data: Option<crate::world::WorldZone>,
    pub adjacent_zones: std::collections::HashMap<crate::world::ZoneCoord, crate::world::WorldZone>, // Store adjacent zones for seamless world view
    pub messages: Vec<String>,
    /// Explored tiles - tracks which tiles have been seen
    /// Key: (zone_x, zone_y), Value: HashSet of (local_x, local_y)
    pub explored_tiles: std::collections::HashMap<(i32, i32), std::collections::HashSet<(i32, i32)>>,
    /// Vision radius for the current character
    pub vision_radius: i32,
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

#[derive(Debug, Clone, PartialEq)]
pub enum CombatPhase {
    // D&D-style turn-based combat (NEW)
    ShowingInitiative,     // Display initiative rolls popup (press Enter to continue)
    PlayerTurn,            // Player's turn - can move and take action freely
    PlayerMovement,        // Player can move with arrow keys before opening action menu
    ResolvingAction,       // Executing a confirmed action
    AITurn,                // AI taking its turn
    CombatComplete(bool),  // Combat over, true if player won

    // Legacy phases (for backwards compatibility with existing combat code)
    InitiativeRoll,
    DeclaringActions,
    SelectingSkill,
    SelectingTarget,
    ResolvingActions,
    RoundComplete,
    PlayerSelectingAction,
    PlayerTargeting,
    ExecutingPlayerAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatPanel {
    Battlefield,
    Movement,
    Combat,
    Skills,
    CharacterInfo,
    TargetInfo,
    SkillsAvailable,
    Inventory,
    SpellDetails,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationMode {
    PanelNavigation, // Shift+HJKL - navigating between panels
    WithinPanel,     // HJKL - navigating within active panel
    Movement,        // WASD - moving player on battlefield
}

#[derive(Debug, Clone)]
pub struct CombatPanelSelections {
    pub movement_index: usize,
    pub combat_index: usize,
    pub skills_index: usize,
    pub character_info_index: usize,
    pub target_info_index: usize,
    pub skills_available_index: usize,
    pub inventory_index: usize,
    pub spell_details_index: usize,
}

// New simplified tactical combat state for redesigned UI
#[derive(Debug, Clone)]
pub struct TacticalCombatState {
    // Core combat data
    pub battlefield: crate::forge::TacticalBattlefield,
    pub participants: Vec<crate::forge::TacticalCombatParticipant>,
    pub initiative_order: Vec<(usize, u8)>, // (participant_index, initiative_roll)
    pub current_turn_index: usize,
    pub round_number: u32,

    // UI state
    pub cursor_pos: (i32, i32), // Cursor position on battlefield
    pub command_buffer: String, // MUD-style command input
    pub combat_log: Vec<String>, // Scrollable combat log
    pub log_scroll_offset: usize,

    // Action state
    pub pending_action: Option<PendingAction>,
    pub available_targets: Vec<usize>, // Indices of valid targets for current action

    // Defensive stance tracking
    pub defending_participants: Vec<usize>, // Indices of participants in defensive stance this round

    // Combat end state
    pub combat_ended_next_state: Option<Box<UIState>>, // If Some, combat has ended - press any key to transition

    // Defeated enemies tracking
    pub defeated_enemy_names: Vec<String>, // Names of defeated non-player participants

    // Return state
    pub return_to_dungeon: Option<DungeonExplorationState>,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    Attack { target_index: Option<usize> },
    Move { destination: Option<(i32, i32)> },
    CastSpell { spell_index: usize, target_index: Option<usize> },
    UseItem { item_index: usize, target_index: Option<usize> },
    Defend,
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
pub struct InventoryState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub view_mode: InventoryViewMode,
    pub sort_mode: InventorySortMode,
    pub filter_type: Option<crate::forge::ItemType>,
    pub showing_details: bool,
    pub selected_item_details: Option<InventoryItemDetails>,
    pub sorted_indices: Vec<usize>, // Maps display index -> original index
}

#[derive(Debug, Clone)]
pub enum InventoryViewMode {
    List,           // Simple list view
    Grid,           // Grid view with icons
    Details,        // Detailed view with stats
}

#[derive(Debug, Clone)]
pub enum InventorySortMode {
    Name,
    Type,
    Weight,
    Value,
    Quantity,
}

#[derive(Debug, Clone)]
pub struct InventoryItemDetails {
    pub item: crate::forge::InventoryItem,
    pub can_equip: bool,
    pub stat_comparison: Option<StatComparison>,
}

#[derive(Debug, Clone)]
pub struct StatComparison {
    pub current_stats: String,
    pub new_stats: String,
    pub improvement: bool,
}

#[derive(Debug, Clone)]
pub struct EquipmentState {
    pub selected_slot: EquipmentSlot,
    pub showing_details: bool,
    pub available_items: Vec<crate::forge::InventoryItem>,
    pub selected_item_index: usize,
}

#[derive(Debug, Clone)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Shield,
    Accessory1,
    Accessory2,
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

impl InventoryState {
    // Helper function to compute sorted indices based on current sort mode
    pub fn compute_sorted_indices(&self, items: &[crate::forge::InventoryItem]) -> Vec<usize> {
        let mut sorted_items: Vec<(usize, &crate::forge::InventoryItem)> = items
            .iter()
            .enumerate()
            .collect();
        
        // Sort based on current sort mode
        match self.sort_mode {
            InventorySortMode::Name => {
                sorted_items.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
            }
            InventorySortMode::Type => {
                sorted_items.sort_by(|(_, a), (_, b)| {
                    let a_type = match &a.item_type {
                        crate::forge::ItemType::Weapon(_) => "Weapon",
                        crate::forge::ItemType::Armor(_) => "Armor", 
                        crate::forge::ItemType::Accessory(_) => "Accessory",
                        crate::forge::ItemType::Consumable(_) => "Consumable",
                        crate::forge::ItemType::Material(_) => "Material",
                        crate::forge::ItemType::Misc(_) => "Misc",
                    };
                    let b_type = match &b.item_type {
                        crate::forge::ItemType::Weapon(_) => "Weapon",
                        crate::forge::ItemType::Armor(_) => "Armor",
                        crate::forge::ItemType::Accessory(_) => "Accessory", 
                        crate::forge::ItemType::Consumable(_) => "Consumable",
                        crate::forge::ItemType::Material(_) => "Material",
                        crate::forge::ItemType::Misc(_) => "Misc",
                    };
                    a_type.cmp(b_type).then(a.name.cmp(&b.name))
                });
            }
            InventorySortMode::Weight => {
                sorted_items.sort_by(|(_, a), (_, b)| {
                    let a_weight = a.weight * a.quantity as f32;
                    let b_weight = b.weight * b.quantity as f32;
                    b_weight.partial_cmp(&a_weight).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            InventorySortMode::Value => {
                sorted_items.sort_by(|(_, a), (_, b)| {
                    let a_value = a.value * a.quantity;
                    let b_value = b.value * b.quantity;
                    b_value.cmp(&a_value)
                });
            }
            InventorySortMode::Quantity => {
                sorted_items.sort_by(|(_, a), (_, b)| b.quantity.cmp(&a.quantity));
            }
        }
        
        // Return the mapping from display index to original index
        sorted_items.iter().map(|(original_index, _)| *original_index).collect()
    }
    
    // Helper to get original index from display index
    pub fn get_original_index(&self, display_index: usize, items: &[crate::forge::InventoryItem]) -> Option<usize> {
        if self.sorted_indices.is_empty() {
            // If no cached sorted indices, compute them
            let sorted_indices = self.compute_sorted_indices(items);
            sorted_indices.get(display_index).copied()
        } else {
            self.sorted_indices.get(display_index).copied()
        }
    }
}

impl TacticalCombatState {
    pub fn new(
        battlefield: crate::forge::TacticalBattlefield,
        participants: Vec<crate::forge::TacticalCombatParticipant>,
        return_to_dungeon: Option<DungeonExplorationState>,
    ) -> Self {
        // Roll initiative for all participants
        let mut initiative_order: Vec<(usize, u8)> = participants
            .iter()
            .enumerate()
            .map(|(idx, _p)| {
                let roll = rand::thread_rng().gen_range(1..=20) as u8;
                // TODO: Add initiative bonus once we have access to full character stats
                (idx, roll)
            })
            .collect();

        // Sort by initiative (highest first)
        // TODO: Add dexterity tiebreaker once we have access to full character stats
        initiative_order.sort_by(|a, b| b.1.cmp(&a.1));

        // Initialize combat log
        let mut combat_log = vec![
            "=== COMBAT BEGINS ===".to_string(),
            "Initiative Order:".to_string(),
        ];
        for (idx, init) in &initiative_order {
            let p = &participants[*idx];
            combat_log.push(format!("  {} ({})", p.base_participant.name, init));
        }
        combat_log.push("".to_string());

        // Start cursor at first player position
        let cursor_pos = if let Some(first_player_idx) = initiative_order.iter().find(|(idx, _)| participants[*idx].base_participant.is_player) {
            battlefield.participant_positions.get(&first_player_idx.0)
                .map(|pos| (pos.x, pos.y))
                .unwrap_or((battlefield.width as i32 / 2, battlefield.height as i32 / 2))
        } else {
            (battlefield.width as i32 / 2, battlefield.height as i32 / 2)
        };

        Self {
            battlefield,
            participants,
            initiative_order,
            current_turn_index: 0,
            round_number: 1,
            cursor_pos,
            command_buffer: String::new(),
            combat_log,
            log_scroll_offset: 0,
            pending_action: None,
            available_targets: Vec::new(),
            defending_participants: Vec::new(),
            combat_ended_next_state: None,
            defeated_enemy_names: Vec::new(),
            return_to_dungeon,
        }
    }

    pub fn get_current_participant(&self) -> Option<&crate::forge::TacticalCombatParticipant> {
        let (participant_idx, _) = self.initiative_order.get(self.current_turn_index)?;
        self.participants.get(*participant_idx)
    }

    pub fn add_log(&mut self, message: String) {
        self.combat_log.push(message);
        // Auto-scroll to bottom
        self.log_scroll_offset = self.combat_log.len().saturating_sub(8);
    }
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
        execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)
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
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show, DisableMouseCapture)?;
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
                UIState::CharacterCreation(creation_state) => Self::draw_character_creation_static(f, creation_state, &input_clone),
                UIState::CharacterList(character_list, selected_index) => Self::draw_character_list_static(f, Some(character_list), *selected_index),
                UIState::Playing => Self::draw_game_static(f, character_clone.as_ref()),
                UIState::CharacterMenu => Self::draw_character_menu_static(f, character_clone.as_ref()),
                UIState::CharacterSheet => Self::draw_character_sheet_static(f, character_clone.as_ref()),
                UIState::InventoryManagement(inventory_state) => Self::draw_inventory_static(f, inventory_state, character_clone.as_ref()),
                UIState::EquipmentManagement(equipment_state) => Self::draw_equipment_static(f, equipment_state, character_clone.as_ref()),
                UIState::WorldExploration(world_state) => Self::draw_world_exploration_static(f, world_state, character_clone.as_ref()),
                UIState::DungeonExploration(dungeon_state) => Self::draw_dungeon_exploration_static(f, dungeon_state, character_clone.as_ref()),
                UIState::Combat(_) => {
                    // Legacy combat state - should not be rendered anymore
                    // All combat now uses TacticalCombat
                    let error_text = Paragraph::new("Error: Legacy combat state encountered")
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center);
                    f.render_widget(error_text, f.size());
                },
                UIState::TacticalCombat(tactical_state) => Self::draw_tactical_combat_static(f, tactical_state, character_clone.as_ref()),
            }
        })?;
        Ok(())
    }

    fn draw_welcome_static(f: &mut Frame) {
        let area = f.size();
        let theme = framework::UITheme::forge_theme();

        // Create beautiful ASCII art title with enhanced styling
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
                Constraint::Length(1),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Min(0),
                Constraint::Length(7),
            ])
            .split(area);

        // Title with gradient effect and enhanced border
        let title_lines: Vec<Line> = title_art.iter()
            .map(|line| Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            )))
            .collect();

        let title = Paragraph::new(title_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()
                    .fg(theme.border_accent)
                    .add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.background)))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[1]);

        // Subtitle with decorative elements
        let subtitle_text = vec![
            Line::from(vec![
                Span::styled("═══════════════════════════════════════", Style::default().fg(theme.border_accent)),
            ]),
            Line::from(vec![
                Span::styled("⚔ ", Style::default().fg(theme.primary)),
                Span::styled("A Forge: Out of Chaos Adventure", Style::default()
                    .fg(theme.info)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC)),
                Span::styled(" ⚔", Style::default().fg(theme.primary)),
            ]),
            Line::from(vec![
                Span::styled("═══════════════════════════════════════", Style::default().fg(theme.border_accent)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Terminal RPG • Turn-Based Combat • Character Progression",
                    Style::default().fg(theme.text_secondary)),
            ]),
        ];

        let subtitle = Paragraph::new(subtitle_text)
            .style(Style::default().bg(theme.background))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_primary)));
        f.render_widget(subtitle, chunks[2]);

        // Story intro with enhanced styling
        let story = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("🌟 ", Style::default().fg(theme.warning)),
                Span::styled("From humble farm worker to mighty warlord,",
                    Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("   your destiny awaits in the realm of chaos!",
                    Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🔥 ", Style::default().fg(theme.error)),
                Span::styled("Master the Forge combat system", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("✨ ", Style::default().fg(theme.sp_color)),
                Span::styled("Wield powerful magic and deadly weapons", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(""),
            Line::from(Span::styled("▶ Press any key to continue...",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))),
        ])
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_primary)));
        f.render_widget(story, chunks[4]);
    }

    fn draw_main_menu_static(f: &mut Frame, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        let theme = framework::UITheme::forge_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(area);

        // Title - show character info if logged in
        let title_content = if let Some(character) = current_character {
            vec![
                Line::from(vec![
                    Span::styled("⚔ WARLORDS ", Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)),
                    Span::styled("MAIN MENU", Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Playing as: ", Style::default().fg(theme.text_secondary)),
                    Span::styled(&character.name, Style::default()
                        .fg(theme.text_highlight)
                        .add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" | Level {} {}", character.level, character.race.name),
                        Style::default().fg(theme.text_secondary)),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("⚔ WARLORDS ", Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)),
                    Span::styled("MAIN MENU", Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD)),
                ]),
            ]
        };

        let title = Paragraph::new(title_content)
            .style(Style::default().bg(theme.background))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()
                    .fg(theme.border_accent)
                    .add_modifier(Modifier::BOLD)));
        f.render_widget(title, chunks[0]);

        // Menu options - different based on whether character is logged in
        let menu_items = if current_character.is_some() {
            vec![
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("1", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • Return to Game World", Style::default()
                            .fg(theme.text_primary)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("2", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • Explore the World", Style::default()
                            .fg(theme.text_primary)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("3", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • Character Menu", Style::default()
                            .fg(theme.text_primary)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("4", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • Logout & Switch Character", Style::default()
                            .fg(theme.text_primary)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("5", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • Quit", Style::default()
                            .fg(theme.error)),
                    ]),
                ]),
                ListItem::new(""),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("▶ ", Style::default().fg(theme.success)),
                        Span::styled("Select an option (1-5):", Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD)),
                    ]),
                ]),
            ]
        } else {
            vec![
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("1", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • ", Style::default()
                            .fg(theme.text_primary)),
                        Span::styled("Login to Existing Character", Style::default()
                            .fg(theme.text_primary)
                            .add_modifier(Modifier::BOLD)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("2", Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • ", Style::default()
                            .fg(theme.success)),
                        Span::styled("Create New Character", Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" ⭐", Style::default()
                            .fg(theme.warning)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled("New to Warlords? Start here!", Style::default()
                            .fg(theme.text_secondary)
                            .add_modifier(Modifier::ITALIC)),
                    ]),
                ]),
                ListItem::new(""),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("3", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • List Characters", Style::default()
                            .fg(theme.text_primary)),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("4", Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)),
                        Span::styled(" • Quit", Style::default()
                            .fg(theme.error)),
                    ]),
                ]),
                ListItem::new(""),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("▶ ", Style::default().fg(theme.success)),
                        Span::styled("Select an option (1-4):", Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD)),
                    ]),
                ]),
            ]
        };

        let menu = List::new(menu_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_primary)))
            .style(Style::default().bg(theme.background));
        f.render_widget(menu, chunks[1]);

        // Instructions with icons
        let instructions = if current_character.is_some() {
            vec![
                Line::from(vec![
                    Span::styled("⌨ ", Style::default().fg(theme.info)),
                    Span::styled("Enter your choice and press ENTER  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("│ ", Style::default().fg(theme.border_secondary)),
                    Span::styled(" M", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Back to Game  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("│ ", Style::default().fg(theme.border_secondary)),
                    Span::styled(" Q/Ctrl+C", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Quit", Style::default().fg(theme.text_secondary)),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("⌨ ", Style::default().fg(theme.info)),
                    Span::styled("Enter your choice and press ENTER  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("│ ", Style::default().fg(theme.border_secondary)),
                    Span::styled(" Q/Ctrl+C", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Quit", Style::default().fg(theme.text_secondary)),
                ]),
            ]
        };

        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().bg(theme.background))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_secondary)));
        f.render_widget(instructions_widget, chunks[2]);
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
            
            // Check if this skill is a racial starting skill
            let is_racial_bonus = if let Some(race) = &creation_state.selected_race {
                race.starting_skills.iter().any(|(race_skill, _)| race_skill == skill)
            } else {
                false
            };
            
            // Check if this skill is available due to race
            let is_racial_exclusive = if let Some(race) = &creation_state.selected_race {
                match race.name.as_str() {
                    "Dwarf" => matches!(skill.as_str(), "Smithing" | "Mining" | "Stone Working"),
                    "Elf" => matches!(skill.as_str(), "Nature Lore" | "Elven Blade Dancing"),
                    "Berserker" => matches!(skill.as_str(), "Berserker Rage" | "Battle Fury"),
                    "Higmoni" => matches!(skill.as_str(), "Desert Survival" | "Heat Resistance"),
                    "Jher-em" => matches!(skill.as_str(), "Telepathy" | "Mental Contact"),
                    "Kithsara" => matches!(skill.as_str(), "Nature Magic" | "Plant Lore"),
                    "Merikii" => matches!(skill.as_str(), "Beast Speech" | "Animal Empathy"),
                    "Sprite" => matches!(skill.as_str(), "Flight" | "Size Change"),
                    "Ghantu" => matches!(skill.as_str(), "Brawling" | "Thick Skin"),
                    _ => false,
                }
            } else {
                false
            };
            
            let (style, skill_display) = if is_current {
                let base_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
                let display = if is_racial_bonus {
                    format!("{} [Racial Start]", skill)
                } else if is_racial_exclusive {
                    format!("{} [Racial]", skill)
                } else {
                    skill.clone()
                };
                (base_style, display)
            } else if selected {
                (Style::default().fg(Color::Green), skill.clone())
            } else if is_racial_bonus {
                (Style::default().fg(Color::LightBlue).add_modifier(Modifier::ITALIC), 
                 format!("{} [Racial Start]", skill))
            } else if is_racial_exclusive {
                (Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC), 
                 format!("{} [Racial]", skill))
            } else {
                (Style::default(), skill.clone())
            };
            
            let prefix = if selected { "✓ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, skill_display)).style(style)
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
                    Line::from("↑/↓ or W/S or K/J: Select character"),
                    Line::from("ENTER: Play selected character"),
                    Line::from("C or N: Create new character"),
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
            Paragraph::new("↑↓/WS/JK: Navigate | ENTER: Play Character | ESC: Main Menu | Q/Ctrl+C: Quit")
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

            for item in &character.inventory.items {
                let quantity_text = if item.quantity > 1 {
                    format!(" ({})", item.quantity)
                } else {
                    String::new()
                };
                details.push(Line::from(format!("• {}{}", item.name, quantity_text)));
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
            let controls = Paragraph::new("I: Inventory | E: Equipment | C: Character Sheet | ESC/M: Return to Game | Q/Ctrl+C: Quit")
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

    fn draw_character_sheet_static(f: &mut Frame, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        let theme = framework::UITheme::forge_theme();

        if let Some(character) = current_character {
            // Main layout: character sheet sections
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),   // Title
                    Constraint::Min(0),      // Sheet content
                    Constraint::Length(4),   // Controls
                ])
                .split(area);

            // Enhanced title with character info
            let title_content = vec![
                Line::from(vec![
                    Span::styled("📋 ", Style::default().fg(theme.info)),
                    Span::styled("CHARACTER SHEET", Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(&character.name, Style::default()
                        .fg(theme.text_highlight)
                        .add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" • Level {} {}", character.level, character.race.name),
                        Style::default().fg(theme.text_secondary)),
                ]),
            ];

            let title = Paragraph::new(title_content)
                .style(Style::default().bg(theme.background))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default()
                        .fg(theme.border_accent)
                        .add_modifier(Modifier::BOLD)));
            f.render_widget(title, main_chunks[0]);

            // Main sheet layout: 3 columns
            let sheet_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33), // Left: Basic info, characteristics
                    Constraint::Percentage(34), // Middle: Combat, skills, magic
                    Constraint::Percentage(33), // Right: Equipment, inventory
                ])
                .split(main_chunks[1]);

            // Left column layout
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(12), // Basic character info
                    Constraint::Length(12), // Characteristics
                    Constraint::Min(0),     // Race info & special abilities
                ])
                .split(sheet_chunks[0]);

            // Middle column layout
            let middle_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // Combat stats
                    Constraint::Length(8),  // Derived values
                    Constraint::Length(10), // Skills
                    Constraint::Min(0),     // Magic
                ])
                .split(sheet_chunks[1]);

            // Right column layout
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(12), // Equipment
                    Constraint::Length(8),  // Encumbrance & movement
                    Constraint::Min(0),     // Inventory
                ])
                .split(sheet_chunks[2]);

            // Draw Basic Character Information
            Self::draw_basic_info_panel(f, left_chunks[0], character);
            
            // Draw Characteristics
            Self::draw_characteristics_panel(f, left_chunks[1], character);
            
            // Draw Race & Special Abilities
            Self::draw_race_abilities_panel(f, left_chunks[2], character);
            
            // Draw Combat Statistics
            Self::draw_combat_stats_panel(f, middle_chunks[0], character);
            
            // Draw Derived Values
            Self::draw_derived_values_panel(f, middle_chunks[1], character);
            
            // Draw Skills
            Self::draw_character_skills_panel(f, middle_chunks[2], character);
            
            // Draw Magic
            Self::draw_magic_panel(f, middle_chunks[3], character);
            
            // Draw Equipment
            Self::draw_equipment_sheet_panel(f, right_chunks[0], character);
            
            // Draw Movement & Encumbrance
            Self::draw_movement_encumbrance_panel(f, right_chunks[1], character);
            
            // Draw Inventory
            Self::draw_inventory_sheet_panel(f, right_chunks[2], character);

            // Enhanced controls
            let controls = vec![
                Line::from(vec![
                    Span::styled("⌨ ", Style::default().fg(theme.info)),
                    Span::styled("ESC/M", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Menu  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("│ ", Style::default().fg(theme.border_secondary)),
                    Span::styled(" I", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Inventory  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("│ ", Style::default().fg(theme.border_secondary)),
                    Span::styled(" E", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Equipment  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("│ ", Style::default().fg(theme.border_secondary)),
                    Span::styled(" Q", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(": Quit", Style::default().fg(theme.text_secondary)),
                ]),
            ];

            let controls_widget = Paragraph::new(controls)
                .style(Style::default().bg(theme.background))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .border_style(Style::default().fg(theme.border_secondary)));
            f.render_widget(controls_widget, main_chunks[2]);
        } else {
            let no_char = Paragraph::new("No character loaded.")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Character Sheet").border_style(Style::default().fg(Color::Red)));
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
            Line::from("↑↓←→/WASD/HJKL: Move | M: Menu | F: Fight | Shift-L: Light | Q: Quit"),
            Line::from("E: Enter/Examine | P: POIs | T: Talk | R: Search | I: Interact | C: Camp | G: Gather"),
        ];
        let controls = Paragraph::new(controls_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Controls").border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(controls, left_chunks[2]);
    }

    /// Apply fog of war dimming effect to a span if it's not currently visible
    fn apply_fog_of_war(span: Span<'static>, is_visible: bool) -> Span<'static> {
        if is_visible {
            span
        } else {
            // Dim the color for explored but not currently visible tiles
            // Use darker versions of colors to preserve the colorful world while maintaining fog of war
            let dimmed_style = match span.style.fg {
                // Reds: keep red tones but dimmed
                Some(Color::Red) | Some(Color::LightRed) => span.style.fg(Color::Red),

                // Yellows: use darker yellow tones
                Some(Color::Yellow) | Some(Color::LightYellow) => span.style.fg(Color::Yellow),

                // Greens: keep green but darker
                Some(Color::Green) | Some(Color::LightGreen) => span.style.fg(Color::Green),

                // Blues and Cyans: preserve blue tones
                Some(Color::Blue) | Some(Color::LightBlue) => span.style.fg(Color::Blue),
                Some(Color::Cyan) | Some(Color::LightCyan) => span.style.fg(Color::Cyan),

                // Magentas: keep purple/magenta tones
                Some(Color::Magenta) | Some(Color::LightMagenta) => span.style.fg(Color::Magenta),

                // Whites and Grays: dim to gray
                Some(Color::White) => span.style.fg(Color::Gray),
                Some(Color::Gray) => span.style.fg(Color::DarkGray),

                // Default: use dark gray for unknown colors
                _ => span.style.fg(Color::DarkGray),
            };
            Span::styled(span.content, dimmed_style.remove_modifier(Modifier::BOLD))
        }
    }

    fn generate_world_view(world_state: &WorldExplorationState, view_width: i32, view_height: i32) -> Vec<Line<'static>> {
        let mut world_content = vec![];
        
        if let Some(zone_data) = &world_state.zone_data {
            // Use full available space for viewport - fill entire world view area
            let actual_view_height = view_height.max(20); // Use full available height
            let actual_view_width = view_width.max(40);   // Use full available width
            
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
                    // Check if this is the player's position (should be at center of viewport)
                    if x == world_state.player_local_pos.x && y == world_state.player_local_pos.y {
                        // Player position - bright yellow @ symbol
                        line_spans.push(Span::styled("@", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    } else {
                        // Calculate which zone this tile belongs to for fog of war checking
                        let (fog_zone_x, fog_zone_y, fog_local_x, fog_local_y) = if x < 0 || x >= crate::world::ZONE_SIZE || y < 0 || y >= crate::world::ZONE_SIZE {
                            let zone_offset_x = if x < 0 { -1 } else if x >= crate::world::ZONE_SIZE { 1 } else { 0 };
                            let zone_offset_y = if y < 0 { -1 } else if y >= crate::world::ZONE_SIZE { 1 } else { 0 };

                            let new_local_x = if x < 0 {
                                crate::world::ZONE_SIZE + x
                            } else if x >= crate::world::ZONE_SIZE {
                                x - crate::world::ZONE_SIZE
                            } else {
                                x
                            };

                            let new_local_y = if y < 0 {
                                crate::world::ZONE_SIZE + y
                            } else if y >= crate::world::ZONE_SIZE {
                                y - crate::world::ZONE_SIZE
                            } else {
                                y
                            };

                            (world_state.current_zone.x + zone_offset_x, world_state.current_zone.y + zone_offset_y, new_local_x, new_local_y)
                        } else {
                            (world_state.current_zone.x, world_state.current_zone.y, x, y)
                        };

                        // Check if tile is explored
                        let is_explored = world_state.explored_tiles
                            .get(&(fog_zone_x, fog_zone_y))
                            .map(|tiles| tiles.contains(&(fog_local_x, fog_local_y)))
                            .unwrap_or(false);

                        // Check if tile is currently visible (within vision radius)
                        let dx = x - world_state.player_local_pos.x;
                        let dy = y - world_state.player_local_pos.y;
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        let is_visible = distance <= world_state.vision_radius as f32;

                        if !is_explored {
                            // Unexplored: completely dark
                            line_spans.push(Span::styled(" ", Style::default().fg(Color::Black)));
                            continue;
                        }

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
                        
                        // Handle adjacent zones - try to load actual terrain if available
                        if let Some(adjacent_zone_coord) = zone_coord {
                            if let Some(adjacent_zone_data) = world_state.adjacent_zones.get(&adjacent_zone_coord) {
                                // We have data for this adjacent zone - render actual terrain
                                // Use the same rendering logic but with adjacent zone data
                                let adjacent_zone = adjacent_zone_data;
                                
                                // Check for settlements first
                                let mut found_settlement = false;
                                for settlement in &adjacent_zone.settlements {
                                    if settlement.position.x == local_x && settlement.position.y == local_y {
                                        found_settlement = true;
                                        let span = match settlement.settlement_type {
                                            crate::world::SettlementType::Capital => Span::styled("█", Style::default().fg(Color::Magenta)),
                                            crate::world::SettlementType::City => Span::styled("●", Style::default().fg(Color::Cyan)),
                                            crate::world::SettlementType::Town => Span::styled("○", Style::default().fg(Color::White)),
                                            crate::world::SettlementType::Village => Span::styled("◦", Style::default().fg(Color::LightYellow)),
                                            crate::world::SettlementType::Outpost => Span::styled("·", Style::default().fg(Color::Gray)),
                                        };
                                        line_spans.push(Self::apply_fog_of_war(span, is_visible));
                                        break;
                                    }
                                }
                                
                                if !found_settlement {
                                    // Check for roads
                                    let mut found_road = false;
                                    if adjacent_zone.roads.get_road_at(crate::world::LocalCoord::new(local_x, local_y)).is_some() {
                                        found_road = true;
                                        line_spans.push(Self::apply_fog_of_war(Span::styled("═", Style::default().fg(Color::DarkGray)), is_visible));
                                    }

                                    if !found_road {
                                        // Render terrain with slightly faded colors to show it's from adjacent zone
                                        if local_x >= 0 && local_x < crate::world::ZONE_SIZE && local_y >= 0 && local_y < crate::world::ZONE_SIZE {
                                            if let Some(row) = adjacent_zone.terrain.tiles.get(local_y as usize) {
                                                if let Some(tile) = row.get(local_x as usize) {
                                                    let symbol = tile.terrain_type.get_ascii_char();
                                                    let base_color = match tile.terrain_type {
                                                        crate::world::TerrainType::Ocean => Color::Blue,
                                                        crate::world::TerrainType::Lake => Color::Cyan,
                                                        crate::world::TerrainType::River => Color::Cyan,
                                                        crate::world::TerrainType::Swamp => Color::Green,
                                                        crate::world::TerrainType::Desert => Color::Yellow,
                                                        crate::world::TerrainType::Plains => Color::Green,
                                                        crate::world::TerrainType::Grassland => Color::Green,
                                                        crate::world::TerrainType::Forest => Color::Green,
                                                        crate::world::TerrainType::Hill => Color::Yellow,
                                                        crate::world::TerrainType::Mountain => Color::Gray,
                                                        crate::world::TerrainType::Snow => Color::White,
                                                        crate::world::TerrainType::Tundra => Color::DarkGray,
                                                    };
                                                    line_spans.push(Self::apply_fog_of_war(Span::styled(symbol.to_string(), Style::default().fg(base_color)), is_visible));
                                                } else {
                                                    line_spans.push(Self::apply_fog_of_war(Span::styled("·", Style::default().fg(Color::DarkGray)), is_visible));
                                                }
                                            } else {
                                                line_spans.push(Self::apply_fog_of_war(Span::styled("·", Style::default().fg(Color::DarkGray)), is_visible));
                                            }
                                        } else {
                                            line_spans.push(Self::apply_fog_of_war(Span::styled("·", Style::default().fg(Color::DarkGray)), is_visible));
                                        }
                                    }
                                }
                            } else {
                                // No data for this adjacent zone - show placeholder
                                line_spans.push(Self::apply_fog_of_war(Span::styled("·", Style::default().fg(Color::DarkGray)), is_visible));
                            }
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
                                let span = match settlement.settlement_type {
                                    crate::world::SettlementType::Capital => Span::styled("█", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                                    crate::world::SettlementType::City => Span::styled("●", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                                    crate::world::SettlementType::Town => Span::styled("○", Style::default().fg(Color::White)),
                                    crate::world::SettlementType::Village => Span::styled("◦", Style::default().fg(Color::LightYellow)),
                                    crate::world::SettlementType::Outpost => Span::styled("·", Style::default().fg(Color::Gray)),
                                };
                                line_spans.push(Self::apply_fog_of_war(span, is_visible));
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
                                    line_spans.push(Self::apply_fog_of_war(Span::styled(npc.npc_type.get_ascii_char().to_string(), Style::default().fg(npc_color)), is_visible));
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
                                        line_spans.push(Self::apply_fog_of_war(Span::styled(symbol.to_string(), Style::default().fg(color)), is_visible));
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
                                                line_spans.push(Self::apply_fog_of_war(Span::styled("═", Style::default().fg(road_color)), is_visible));
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
                                                
                                                line_spans.push(Self::apply_fog_of_war(Span::styled(symbol.to_string(), style), is_visible));
                                            } else {
                                                line_spans.push(Self::apply_fog_of_war(Span::styled("?", Style::default().fg(Color::Red)), is_visible));
                                            }
                                        } else {
                                            line_spans.push(Self::apply_fog_of_war(Span::styled("?", Style::default().fg(Color::Red)), is_visible));
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

        // Tactical combat is now always standalone UIState::TacticalCombat
        // No need to check for embedded combat here
        
        // Normal dungeon exploration layout
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
        let controls = Paragraph::new("↑↓←→/WASD/HJKL: Move | E: Examine | I: Interact | F: Fight | U: Stairs | Shift-L: Light | X: Exit | Ctrl+Q: Quit")
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
                Line::from(format!("Gold: {}", character.gold)),
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

    fn draw_tactical_combat_static(f: &mut Frame, tactical_state: &TacticalCombatState, _current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();

        // Main layout: Top 60% for battlefield area, Bottom 40% for log/actions/input
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),  // Top: Battlefield + Initiative sidebar
                Constraint::Percentage(40),  // Bottom: Combat log + Actions + Input
            ])
            .split(area);

        // Top area: Split into battlefield (left 70%) and initiative sidebar (right 30%)
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),  // Battlefield
                Constraint::Percentage(30),  // Initiative + Quick Stats
            ])
            .split(main_chunks[0]);

        // Bottom area: Combat log (8 lines), action bar (1 line), command input (1 line)
        let bottom_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),        // Combat log (scrollable)
                Constraint::Length(3),     // Action hotkey bar
                Constraint::Length(3),     // Command input line
            ])
            .split(main_chunks[1]);

        // === BATTLEFIELD PANEL (Top Left) ===
        Self::draw_battlefield_panel(f, top_chunks[0], tactical_state);

        // === INITIATIVE SIDEBAR (Top Right) ===
        Self::draw_initiative_sidebar_panel(f, top_chunks[1], tactical_state);

        // === COMBAT LOG (Bottom Top) ===
        Self::draw_combat_log_panel(f, bottom_chunks[0], tactical_state);

        // === ACTION HOTKEY BAR (Bottom Middle) ===
        Self::draw_action_hotkey_bar(f, bottom_chunks[1], tactical_state);

        // === COMMAND INPUT (Bottom Bottom) ===
        Self::draw_command_input_panel(f, bottom_chunks[2], tactical_state);
    }

    fn draw_battlefield_panel(f: &mut Frame, area: ratatui::layout::Rect, tactical_state: &TacticalCombatState) {
        let battlefield = &tactical_state.battlefield;
        let cursor_pos = tactical_state.cursor_pos;

        // Calculate the available area inside borders
        let available_height = area.height.saturating_sub(2) as i32; // Subtract borders
        let available_width = area.width.saturating_sub(2) as i32; // Subtract borders

        // Calculate battlefield dimensions
        let battlefield_width = battlefield.width as i32;
        let battlefield_height = battlefield.height as i32;

        // Calculate padding to center the battlefield
        let pad_top = ((available_height - battlefield_height) / 2).max(0);
        let pad_left = ((available_width - battlefield_width) / 2).max(0);

        // Generate battlefield view with centering
        let mut battlefield_lines = Vec::new();

        // Add top padding
        for _ in 0..pad_top {
            battlefield_lines.push(Line::from(" ".repeat(available_width as usize)));
        }

        // Render battlefield rows
        for y in 0..battlefield_height {
            let mut line_spans = Vec::new();

            // Add left padding
            for _ in 0..pad_left {
                line_spans.push(Span::raw(" "));
            }

            // Render battlefield columns
            for x in 0..battlefield_width {
                // Check if cursor is at this position
                if x == cursor_pos.0 && y == cursor_pos.1 {
                    // Show cursor
                    line_spans.push(Span::styled("X", Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::REVERSED)));
                } else {
                    // Check if a participant is at this position
                    let participant_at_pos = tactical_state.participants.iter().enumerate().find(|(idx, _)| {
                        battlefield.participant_positions.get(idx)
                            .map(|pos| pos.x == x && pos.y == y)
                            .unwrap_or(false)
                    });

                    if let Some((idx, participant)) = participant_at_pos {
                        // Render participant
                        let symbol = if participant.base_participant.is_player {
                            // Player: Use number (1, 2, 3...) colored by class
                            let player_num = tactical_state.participants.iter()
                                .enumerate()
                                .filter(|(_, p)| p.base_participant.is_player)
                                .position(|(i, _)| i == idx)
                                .unwrap_or(0) + 1;

                            // Color by class (for now, use cyan for all players)
                            Span::styled(
                                player_num.to_string(),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                            )
                        } else {
                            // Enemy: Use first letter of name, colored by difficulty tier
                            let first_letter = participant.base_participant.name.chars().next().unwrap_or('?');

                            // Difficulty tier color (gray → light green → blue → yellow → orange → red → purple)
                            // For now, use red for enemies (TODO: Calculate based on level comparison)
                            Span::styled(
                                first_letter.to_string(),
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            )
                        };

                        line_spans.push(symbol);
                    } else {
                        // Empty tile - check terrain
                        let position = crate::forge::BattlefieldPosition::new(x, y);
                        let terrain = battlefield.tiles.get(&position)
                            .map(|tile| tile.terrain.clone())
                            .unwrap_or(crate::forge::TerrainFeature::Open);

                        let (symbol, color) = match terrain {
                            crate::forge::TerrainFeature::Open => ('·', Color::DarkGray),
                            crate::forge::TerrainFeature::Obstacle => ('▓', Color::Gray),
                            crate::forge::TerrainFeature::DifficultTerrain => ('≋', Color::Yellow),
                            crate::forge::TerrainFeature::Cover => ('▒', Color::Green),
                            crate::forge::TerrainFeature::Hazard => ('※', Color::Red),
                            crate::forge::TerrainFeature::Elevation => ('▲', Color::LightYellow),
                            crate::forge::TerrainFeature::Water => ('≈', Color::Blue),
                            crate::forge::TerrainFeature::Altar => ('✦', Color::Magenta),
                            crate::forge::TerrainFeature::Pillar => ('●', Color::Gray),
                            crate::forge::TerrainFeature::Pit => ('◎', Color::Red),
                        };

                        line_spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
                    }
                }
            }

            // Add right padding
            let remaining = available_width - pad_left - battlefield_width;
            for _ in 0..remaining {
                line_spans.push(Span::raw(" "));
            }

            battlefield_lines.push(Line::from(line_spans));
        }

        // Add bottom padding
        let remaining_bottom = available_height - pad_top - battlefield_height;
        for _ in 0..remaining_bottom {
            battlefield_lines.push(Line::from(" ".repeat(available_width as usize)));
        }

        // Create decorative battlefield title with ASCII art
        let battlefield_title = vec![
            Span::styled("╡ ", Style::default().fg(Color::Green)),
            Span::styled("⚔", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" BATTLEFIELD ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("⚔", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" ╞", Style::default().fg(Color::Green)),
        ];

        let battlefield_widget = Paragraph::new(battlefield_lines)
            .block(Block::default()
                .title(Line::from(battlefield_title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::default().fg(Color::Green)));

        f.render_widget(battlefield_widget, area);
    }

    fn draw_initiative_sidebar_panel(f: &mut Frame, area: ratatui::layout::Rect, tactical_state: &TacticalCombatState) {
        // Split sidebar into Initiative Order (top) and Quick Stats (bottom)
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),  // Initiative order
                Constraint::Percentage(40),  // Quick stats
            ])
            .split(area);

        // === INITIATIVE ORDER ===
        let mut init_lines = vec![
            Line::from(vec![
                Span::styled("═══", Style::default().fg(Color::Yellow)),
                Span::styled("╣ ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("Round {}", tactical_state.round_number), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" ╠", Style::default().fg(Color::Yellow)),
                Span::styled("═══", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
        ];

        for (idx, (participant_idx, init_roll)) in tactical_state.initiative_order.iter().enumerate() {
            let participant = &tactical_state.participants[*participant_idx];
            let is_current = idx == tactical_state.current_turn_index;

            let symbol = if participant.base_participant.is_player {
                let player_num = tactical_state.participants.iter()
                    .enumerate()
                    .filter(|(_, p)| p.base_participant.is_player)
                    .position(|(i, _)| i == *participant_idx)
                    .unwrap_or(0) + 1;
                player_num.to_string()
            } else {
                participant.base_participant.name.chars().next().unwrap_or('?').to_string()
            };

            let color = if participant.base_participant.is_player {
                Color::Cyan
            } else {
                Color::Red
            };

            let marker = if is_current { "» " } else { "  " };
            let style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            init_lines.push(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(symbol, style.add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", participant.base_participant.name), style),
                Span::styled(format!("({})", init_roll), Style::default().fg(Color::DarkGray)),
            ]));
        }

        let initiative_title = vec![
            Span::styled("╡ ", Style::default().fg(Color::Yellow)),
            Span::styled("⚡", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" INITIATIVE ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("⚡", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ╞", Style::default().fg(Color::Yellow)),
        ];

        let initiative_widget = Paragraph::new(init_lines)
            .block(Block::default()
                .title(Line::from(initiative_title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::default().fg(Color::Yellow)));

        f.render_widget(initiative_widget, sidebar_chunks[0]);

        // === QUICK STATS ===
        let current_participant = tactical_state.get_current_participant();
        let stats_lines = if let Some(participant) = current_participant {
            if participant.base_participant.is_player {
                // Show full stats for player characters
                let hp_current = participant.base_participant.combat_stats.hit_points.current;
                let hp_max = participant.base_participant.combat_stats.hit_points.max;
                let hp_percent = (hp_current as f32 / hp_max as f32 * 100.0) as u32;
                let hp_color = if hp_percent >= 75 { Color::Green } else if hp_percent >= 50 { Color::Yellow } else if hp_percent >= 25 { Color::LightRed } else { Color::Red };

                vec![
                    Line::from(vec![
                        Span::styled("╔═══ ", Style::default().fg(Color::Cyan)),
                        Span::styled(participant.base_participant.name.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(" ═══╗", Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("♥ ", Style::default().fg(hp_color).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{}/{}", hp_current, hp_max), Style::default().fg(hp_color)),
                    ]),
                    Line::from(vec![
                        Span::styled("⚔ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("AV: {}", participant.base_participant.combat_stats.attack_value), Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::styled("◈ ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("DV: {}", participant.base_participant.combat_stats.defensive_value), Style::default().fg(Color::White)),
                    ]),
                ]
            } else {
                // Show threat level for enemies instead of exact stats
                let (threat_level, threat_color) = {
                    // For now, use red for all enemies (TODO: Calculate based on level comparison)
                    ("Dangerous", Color::Red)
                };

                // Calculate health bar representation without showing exact numbers
                let hp_current = participant.base_participant.combat_stats.hit_points.current;
                let hp_max = participant.base_participant.combat_stats.hit_points.max;
                let hp_percent = (hp_current as f32 / hp_max as f32 * 100.0) as u32;

                let health_status = if hp_percent >= 75 {
                    ("Healthy", Color::Green)
                } else if hp_percent >= 50 {
                    ("Wounded", Color::Yellow)
                } else if hp_percent >= 25 {
                    ("Bloodied", Color::LightRed)
                } else {
                    ("Near Death", Color::Red)
                };

                vec![
                    Line::from(vec![
                        Span::styled("╔═══ ", Style::default().fg(Color::Red)),
                        Span::styled(participant.base_participant.name.clone(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled(" ═══╗", Style::default().fg(Color::Red)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("⚠ ", Style::default().fg(threat_color).add_modifier(Modifier::BOLD)),
                        Span::raw("Threat: "),
                        Span::styled(threat_level, Style::default().fg(threat_color).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("♥ ", Style::default().fg(health_status.1).add_modifier(Modifier::BOLD)),
                        Span::raw("Status: "),
                        Span::styled(health_status.0, Style::default().fg(health_status.1)),
                    ]),
                ]
            }
        } else {
            vec![Line::from("No active participant")]
        };

        let stats_title = vec![
            Span::styled("╡ ", Style::default().fg(Color::Cyan)),
            Span::styled("◈", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" STATS ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("◈", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ╞", Style::default().fg(Color::Cyan)),
        ];

        let stats_widget = Paragraph::new(stats_lines)
            .block(Block::default()
                .title(Line::from(stats_title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan)));

        f.render_widget(stats_widget, sidebar_chunks[1]);
    }

    fn draw_combat_log_panel(f: &mut Frame, area: ratatui::layout::Rect, tactical_state: &TacticalCombatState) {
        // Show the last 8 lines of combat log (or whatever fits)
        let visible_lines = area.height.saturating_sub(2) as usize; // Subtract borders
        let start_idx = tactical_state.log_scroll_offset;
        let end_idx = (start_idx + visible_lines).min(tactical_state.combat_log.len());

        let log_lines: Vec<Line> = tactical_state.combat_log[start_idx..end_idx]
            .iter()
            .map(|msg| Line::from(msg.clone()))
            .collect();

        let log_title = vec![
            Span::styled("╡ ", Style::default().fg(Color::Blue)),
            Span::styled("⚔", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" BATTLE LOG ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled("⚔", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ╞", Style::default().fg(Color::Blue)),
        ];

        let log_widget = Paragraph::new(log_lines)
            .block(Block::default()
                .title(Line::from(log_title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue)))
            .wrap(Wrap { trim: true });

        f.render_widget(log_widget, area);
    }

    fn draw_action_hotkey_bar(f: &mut Frame, area: ratatui::layout::Rect, _tactical_state: &TacticalCombatState) {
        let hotkey_text = vec![
            Line::from(vec![
                Span::styled("║ ", Style::default().fg(Color::Green)),
                Span::styled("⚔[A]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("ttack ", Style::default().fg(Color::White)),
                Span::styled("♦[M]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("ove ", Style::default().fg(Color::White)),
                Span::styled("✦[C]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("ast ", Style::default().fg(Color::White)),
                Span::styled("◈[U]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("se ", Style::default().fg(Color::White)),
                Span::styled("◉[D]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("efend ", Style::default().fg(Color::White)),
                Span::styled("★[S]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("kills ", Style::default().fg(Color::White)),
                Span::styled("▶[E]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("nd Turn", Style::default().fg(Color::White)),
                Span::styled(" ║", Style::default().fg(Color::Green)),
            ]),
        ];

        let actions_title = vec![
            Span::styled("╡ ", Style::default().fg(Color::Green)),
            Span::styled("☆", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ACTIONS ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("☆", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ╞", Style::default().fg(Color::Green)),
        ];

        let hotkeys_widget = Paragraph::new(hotkey_text)
            .block(Block::default()
                .title(Line::from(actions_title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)))
            .alignment(Alignment::Center);

        f.render_widget(hotkeys_widget, area);
    }

    fn draw_command_input_panel(f: &mut Frame, area: ratatui::layout::Rect, tactical_state: &TacticalCombatState) {
        let input_text = if tactical_state.command_buffer.is_empty() {
            Line::from(vec![
                Span::styled("» ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Enter command...", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ])
        } else {
            Line::from(vec![
                Span::styled("» ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(&tactical_state.command_buffer, Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::White)),
            ])
        };

        let command_title = vec![
            Span::styled("╡ ", Style::default().fg(Color::Green)),
            Span::styled("⌘", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" COMMAND ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("⌘", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ╞", Style::default().fg(Color::Green)),
        ];

        let input_widget = Paragraph::new(input_text)
            .block(Block::default()
                .title(Line::from(command_title))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)));

        f.render_widget(input_widget, area);
    }

    #[allow(dead_code)]
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
                    _ => {
                        let unknown = Paragraph::new("Unknown combat phase")
                            .style(Style::default().fg(Color::Red))
                            .alignment(Alignment::Center)
                            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)));
                        f.render_widget(unknown, chunks[3]);
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
                _ => {
                    Paragraph::new("Unknown combat phase")
                        .style(Style::default().fg(Color::Red))
                }
            }
        };
        
        let controls = controls
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(controls, chunks[4]);
    }

    pub fn handle_input(&self) -> anyhow::Result<Option<Event>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle Ctrl+C for graceful shutdown
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        return Ok(Some(Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL))));
                    }
                    return Ok(Some(Event::Key(key)));
                }
                Event::Mouse(mouse) => {
                    return Ok(Some(Event::Mouse(mouse)));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn draw_inventory_static(f: &mut Frame, inventory_state: &InventoryState, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        if let Some(character) = current_character {
            // Main layout: inventory list, details panel, controls
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(60), // Item list
                    Constraint::Percentage(40), // Details/stats
                ])
                .split(area);
                
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Item list
                    Constraint::Length(5), // Weight/capacity info
                    Constraint::Length(3), // Controls
                ])
                .split(main_chunks[0]);
                
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Equipment status
                    Constraint::Min(0),     // Item details
                ])
                .split(main_chunks[1]);

            // Enhanced title with sort/filter info
            let theme = framework::UITheme::forge_theme();

            let (sort_text, sort_icon) = match inventory_state.sort_mode {
                InventorySortMode::Name => ("Name", "📝"),
                InventorySortMode::Type => ("Type", "📦"),
                InventorySortMode::Weight => ("Weight", "⚖"),
                InventorySortMode::Value => ("Value", "🪙"),
                InventorySortMode::Quantity => ("Quantity", "🔢"),
            };

            let filter_text = match &inventory_state.filter_type {
                Some(_) => " (Filtered)",
                None => "",
            };

            let title_content = vec![
                Line::from(vec![
                    Span::styled("📦 ", Style::default().fg(theme.info)),
                    Span::styled("INVENTORY", Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format!("{} Sort: ", sort_icon), Style::default().fg(theme.text_secondary)),
                    Span::styled(sort_text, Style::default()
                        .fg(theme.text_highlight)
                        .add_modifier(Modifier::BOLD)),
                    Span::styled(filter_text, Style::default().fg(theme.warning)),
                ]),
            ];

            let title = Paragraph::new(title_content)
                .style(Style::default().bg(theme.background))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default()
                        .fg(theme.border_accent)
                        .add_modifier(Modifier::BOLD)));
            f.render_widget(title, left_chunks[0]);

            // Item list - create sorted version based on current sort mode
            let mut sorted_items: Vec<(usize, &crate::forge::InventoryItem)> = character.inventory.items
                .iter()
                .enumerate()
                .collect();
            
            // Sort based on current sort mode
            match inventory_state.sort_mode {
                InventorySortMode::Name => {
                    sorted_items.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
                }
                InventorySortMode::Type => {
                    sorted_items.sort_by(|(_, a), (_, b)| {
                        let a_type = match &a.item_type {
                            crate::forge::ItemType::Weapon(_) => "Weapon",
                            crate::forge::ItemType::Armor(_) => "Armor", 
                            crate::forge::ItemType::Accessory(_) => "Accessory",
                            crate::forge::ItemType::Consumable(_) => "Consumable",
                            crate::forge::ItemType::Material(_) => "Material",
                            crate::forge::ItemType::Misc(_) => "Misc",
                        };
                        let b_type = match &b.item_type {
                            crate::forge::ItemType::Weapon(_) => "Weapon",
                            crate::forge::ItemType::Armor(_) => "Armor",
                            crate::forge::ItemType::Accessory(_) => "Accessory", 
                            crate::forge::ItemType::Consumable(_) => "Consumable",
                            crate::forge::ItemType::Material(_) => "Material",
                            crate::forge::ItemType::Misc(_) => "Misc",
                        };
                        a_type.cmp(b_type).then(a.name.cmp(&b.name))
                    });
                }
                InventorySortMode::Weight => {
                    sorted_items.sort_by(|(_, a), (_, b)| {
                        let a_weight = a.weight * a.quantity as f32;
                        let b_weight = b.weight * b.quantity as f32;
                        b_weight.partial_cmp(&a_weight).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                InventorySortMode::Value => {
                    sorted_items.sort_by(|(_, a), (_, b)| {
                        let a_value = a.value * a.quantity;
                        let b_value = b.value * b.quantity;
                        b_value.cmp(&a_value)
                    });
                }
                InventorySortMode::Quantity => {
                    sorted_items.sort_by(|(_, a), (_, b)| b.quantity.cmp(&a.quantity));
                }
            }
            
            let mut item_lines = Vec::new();

            for (display_index, (_original_index, item)) in sorted_items.iter().enumerate() {
                let selected = display_index == inventory_state.selected_index;

                // Get icon and color based on item type
                let (icon, type_color) = match &item.item_type {
                    crate::forge::ItemType::Weapon(_) => ("⚔", theme.primary),
                    crate::forge::ItemType::Armor(_) => ("🛡", theme.info),
                    crate::forge::ItemType::Accessory(_) => ("💍", theme.sp_color),
                    crate::forge::ItemType::Consumable(_) => ("🧪", theme.success),
                    crate::forge::ItemType::Material(_) => ("🔧", theme.text_secondary),
                    crate::forge::ItemType::Misc(_) => ("📦", theme.text_muted),
                };

                let quantity_text = if item.quantity > 1 {
                    format!(" ×{}", item.quantity)
                } else {
                    String::new()
                };

                let weight_text = format!(" ⚖{:.1}", item.weight * item.quantity as f32);
                let value_text = format!(" 🪙{}", item.value * item.quantity);

                if selected {
                    item_lines.push(ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(format!("{} ", icon), Style::default()
                                .fg(type_color)
                                .bg(theme.secondary)
                                .add_modifier(Modifier::BOLD)),
                            Span::styled(&item.name, Style::default()
                                .fg(theme.text_highlight)
                                .bg(theme.secondary)
                                .add_modifier(Modifier::BOLD)),
                            Span::styled(quantity_text, Style::default()
                                .fg(theme.accent)
                                .bg(theme.secondary)),
                            Span::styled(weight_text, Style::default()
                                .fg(theme.text_secondary)
                                .bg(theme.secondary)),
                            Span::styled(value_text, Style::default()
                                .fg(theme.warning)
                                .bg(theme.secondary)),
                        ]),
                    ]));
                } else {
                    item_lines.push(ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(format!("{} ", icon), Style::default().fg(type_color)),
                            Span::styled(&item.name, Style::default().fg(theme.text_primary)),
                            Span::styled(quantity_text, Style::default().fg(theme.text_secondary)),
                            Span::styled(weight_text, Style::default().fg(theme.text_muted)),
                            Span::styled(value_text, Style::default().fg(theme.warning)),
                        ]),
                    ]));
                }
            }

            if sorted_items.is_empty() {
                item_lines.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("📭 ", Style::default().fg(theme.text_muted)),
                        Span::styled("No items in inventory", Style::default().fg(theme.text_muted)),
                    ]),
                ]));
            }

            let item_list = List::new(item_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Items")
                    .border_style(Style::default().fg(theme.border_primary)))
                .style(Style::default().bg(theme.background));
            f.render_widget(item_list, left_chunks[1]);

            // Enhanced weight capacity info with progress bar
            let current_weight: f32 = character.inventory.items.iter().map(|item| item.weight * item.quantity as f32).sum();
            let max_weight = character.inventory.max_weight;
            let weight_percentage = (current_weight / max_weight * 100.0) as u8;

            let weight_color = match weight_percentage {
                0..=70 => theme.success,
                71..=90 => theme.warning,
                _ => theme.error,
            };

            let total_value: u32 = character.inventory.items.iter().map(|item| item.value * item.quantity).sum();

            let weight_bar_width = left_chunks[2].width.saturating_sub(6).min(30) as usize;
            let weight_bar = framework::art::progress_bar(
                current_weight as u32,
                max_weight as u32,
                weight_bar_width,
                '█',
                '░'
            );

            let weight_info = vec![
                Line::from(vec![
                    Span::styled("⚖ ", Style::default().fg(weight_color)),
                    Span::styled(format!("{:.1}/{:.1} kg", current_weight, max_weight),
                        Style::default().fg(weight_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" ({}%)", weight_percentage),
                        Style::default().fg(theme.text_secondary)),
                ]),
                Line::from(vec![
                    Span::styled(weight_bar, Style::default().fg(weight_color)),
                ]),
                Line::from(vec![
                    Span::styled("📦 ", Style::default().fg(theme.info)),
                    Span::styled(format!("{} items", character.inventory.items.len()),
                        Style::default().fg(theme.text_primary)),
                    Span::styled("  🪙 ", Style::default().fg(theme.warning)),
                    Span::styled(format!("{}gp", total_value),
                        Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                ]),
            ];
            
            let weight_panel = Paragraph::new(weight_info)
                .style(Style::default().fg(weight_color))
                .block(Block::default().borders(Borders::ALL).title("Capacity").border_style(Style::default().fg(weight_color)));
            f.render_widget(weight_panel, left_chunks[2]);

            // Equipment status with icons
            let equipped_items = vec![
                framework::art::section_header("EQUIPPED", right_chunks[0].width.saturating_sub(4) as usize, &theme),
                Line::from(""),
                Line::from(vec![
                    Span::styled("⚔ ", Style::default().fg(theme.primary)),
                    Span::styled(
                        character.equipment.weapon.as_ref().map(|w| w.name.as_str()).unwrap_or("Empty"),
                        if character.equipment.weapon.is_some() { Style::default().fg(theme.success) } else { Style::default().fg(theme.text_muted) }
                    ),
                ]),
                Line::from(vec![
                    Span::styled("🛡 ", Style::default().fg(theme.info)),
                    Span::styled(
                        character.equipment.armor.as_ref().map(|a| a.name.as_str()).unwrap_or("Empty"),
                        if character.equipment.armor.is_some() { Style::default().fg(theme.success) } else { Style::default().fg(theme.text_muted) }
                    ),
                ]),
                Line::from(vec![
                    Span::styled("🔰 ", Style::default().fg(theme.info)),
                    Span::styled(
                        character.equipment.shield.as_ref().map(|s| s.name.as_str()).unwrap_or("Empty"),
                        if character.equipment.shield.is_some() { Style::default().fg(theme.success) } else { Style::default().fg(theme.text_muted) }
                    ),
                ]),
                Line::from(vec![
                    Span::styled("💍 ", Style::default().fg(Color::Magenta)),
                    Span::styled(
                        character.equipment.accessory1.as_ref().map(|a| a.name.as_str()).unwrap_or("Empty"),
                        if character.equipment.accessory1.is_some() { Style::default().fg(theme.success) } else { Style::default().fg(theme.text_muted) }
                    ),
                ]),
                Line::from(vec![
                    Span::styled("💍 ", Style::default().fg(Color::Magenta)),
                    Span::styled(
                        character.equipment.accessory2.as_ref().map(|a| a.name.as_str()).unwrap_or("Empty"),
                        if character.equipment.accessory2.is_some() { Style::default().fg(theme.success) } else { Style::default().fg(theme.text_muted) }
                    ),
                ]),
            ];

            let equipment_panel = Paragraph::new(equipped_items)
                .block(Block::default().borders(Borders::ALL).title("⚡ Equipment").border_style(Style::default().fg(theme.accent)));
            f.render_widget(equipment_panel, right_chunks[0]);

            // Item details - get selected item from sorted view
            let details = if let Some((_, selected_item)) = sorted_items.get(inventory_state.selected_index) {
                vec![
                    Line::from(Span::styled(&selected_item.name, Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(selected_item.description.as_str()),
                    Line::from(""),
                    Line::from(format!("Weight: {:.1} kg", selected_item.weight)),
                    Line::from(format!("Value: {} gp", selected_item.value)),
                    Line::from(format!("Quantity: {}", selected_item.quantity)),
                    Line::from(""),
                    Line::from(match &selected_item.item_type {
                        crate::forge::ItemType::Weapon(_) => "Type: Weapon",
                        crate::forge::ItemType::Armor(_) => "Type: Armor", 
                        crate::forge::ItemType::Accessory(_) => "Type: Accessory",
                        crate::forge::ItemType::Consumable(_) => "Type: Consumable",
                        crate::forge::ItemType::Material(_) => "Type: Material",
                        crate::forge::ItemType::Misc(_) => "Type: Misc",
                    }),
                ]
            } else {
                vec![Line::from("No item selected")]
            };
            
            let details_panel = Paragraph::new(details)
                .block(Block::default().borders(Borders::ALL).title("Item Details").border_style(Style::default().fg(Color::Magenta)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(details_panel, right_chunks[1]);

            // Controls
            let controls = Paragraph::new("↑↓/WS/JK: Navigate | ENTER: Use/Equip | D: Drop | TAB: Sort | F: Filter | ESC: Back")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Controls").border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(controls, left_chunks[3]);
            
        } else {
            let no_char = Paragraph::new("No character loaded.")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Inventory").border_style(Style::default().fg(Color::Red)));
            f.render_widget(no_char, area);
        }
    }

    fn draw_equipment_static(f: &mut Frame, equipment_state: &EquipmentState, current_character: Option<&crate::forge::ForgeCharacter>) {
        let area = f.size();
        
        if let Some(character) = current_character {
            // Main layout: equipment slots, available items, item details
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40), // Equipment slots
                    Constraint::Percentage(35), // Available items
                    Constraint::Percentage(25), // Item details
                ])
                .split(area);
                
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Min(0),     // Equipment slots
                    Constraint::Length(3),  // Controls
                ])
                .split(main_chunks[0]);

            // Title
            let theme = framework::UITheme::forge_theme();
            let title_content = vec![
                Line::from(vec![
                    Span::styled("⚔ ", Style::default().fg(theme.primary)),
                    Span::styled("EQUIPMENT MANAGEMENT", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(" 🛡", Style::default().fg(theme.info)),
                ]),
                Line::from(vec![
                    Span::styled(&character.name, Style::default().fg(theme.text_highlight)),
                    Span::styled(format!(" • Level {} {}", character.level, character.race.name), Style::default().fg(theme.text_secondary)),
                ]),
            ];
            let title = Paragraph::new(title_content)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
            f.render_widget(title, left_chunks[0]);

            // Equipment slots with icons and color-coding
            let mut slot_lines = Vec::new();

            // Weapon slot
            let weapon_selected = matches!(equipment_state.selected_slot, EquipmentSlot::Weapon);
            let weapon_name = character.equipment.weapon.as_ref().map(|w| w.name.as_str()).unwrap_or("Empty");
            let weapon_color = if character.equipment.weapon.is_some() { theme.success } else { theme.text_muted };
            let line = if weapon_selected {
                Line::from(vec![
                    Span::styled("⚔ ", Style::default().fg(theme.primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled("Weapon: ", Style::default().fg(theme.text_primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled(weapon_name, Style::default().fg(weapon_color).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("⚔ ", Style::default().fg(theme.text_secondary)),
                    Span::styled("Weapon: ", Style::default().fg(theme.text_primary)),
                    Span::styled(weapon_name, Style::default().fg(weapon_color)),
                ])
            };
            slot_lines.push(ListItem::new(line));

            // Armor slot
            let armor_selected = matches!(equipment_state.selected_slot, EquipmentSlot::Armor);
            let armor_name = character.equipment.armor.as_ref().map(|a| a.name.as_str()).unwrap_or("Empty");
            let armor_color = if character.equipment.armor.is_some() { theme.success } else { theme.text_muted };
            let line = if armor_selected {
                Line::from(vec![
                    Span::styled("🛡 ", Style::default().fg(theme.primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled("Armor: ", Style::default().fg(theme.text_primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled(armor_name, Style::default().fg(armor_color).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("🛡 ", Style::default().fg(theme.text_secondary)),
                    Span::styled("Armor: ", Style::default().fg(theme.text_primary)),
                    Span::styled(armor_name, Style::default().fg(armor_color)),
                ])
            };
            slot_lines.push(ListItem::new(line));

            // Shield slot
            let shield_selected = matches!(equipment_state.selected_slot, EquipmentSlot::Shield);
            let shield_name = character.equipment.shield.as_ref().map(|s| s.name.as_str()).unwrap_or("Empty");
            let shield_color = if character.equipment.shield.is_some() { theme.success } else { theme.text_muted };
            let line = if shield_selected {
                Line::from(vec![
                    Span::styled("🔰 ", Style::default().fg(theme.primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled("Shield: ", Style::default().fg(theme.text_primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled(shield_name, Style::default().fg(shield_color).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("🔰 ", Style::default().fg(theme.text_secondary)),
                    Span::styled("Shield: ", Style::default().fg(theme.text_primary)),
                    Span::styled(shield_name, Style::default().fg(shield_color)),
                ])
            };
            slot_lines.push(ListItem::new(line));

            // Accessory 1 slot
            let acc1_selected = matches!(equipment_state.selected_slot, EquipmentSlot::Accessory1);
            let acc1_name = character.equipment.accessory1.as_ref().map(|a| a.name.as_str()).unwrap_or("Empty");
            let acc1_color = if character.equipment.accessory1.is_some() { theme.success } else { theme.text_muted };
            let line = if acc1_selected {
                Line::from(vec![
                    Span::styled("💍 ", Style::default().fg(theme.primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled("Accessory 1: ", Style::default().fg(theme.text_primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled(acc1_name, Style::default().fg(acc1_color).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("💍 ", Style::default().fg(theme.text_secondary)),
                    Span::styled("Accessory 1: ", Style::default().fg(theme.text_primary)),
                    Span::styled(acc1_name, Style::default().fg(acc1_color)),
                ])
            };
            slot_lines.push(ListItem::new(line));

            // Accessory 2 slot
            let acc2_selected = matches!(equipment_state.selected_slot, EquipmentSlot::Accessory2);
            let acc2_name = character.equipment.accessory2.as_ref().map(|a| a.name.as_str()).unwrap_or("Empty");
            let acc2_color = if character.equipment.accessory2.is_some() { theme.success } else { theme.text_muted };
            let line = if acc2_selected {
                Line::from(vec![
                    Span::styled("💍 ", Style::default().fg(theme.primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled("Accessory 2: ", Style::default().fg(theme.text_primary).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled(acc2_name, Style::default().fg(acc2_color).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("💍 ", Style::default().fg(theme.text_secondary)),
                    Span::styled("Accessory 2: ", Style::default().fg(theme.text_primary)),
                    Span::styled(acc2_name, Style::default().fg(acc2_color)),
                ])
            };
            slot_lines.push(ListItem::new(line));

            let slot_list = List::new(slot_lines)
                .block(Block::default().borders(Borders::ALL).title("⚡ Equipment Slots").border_style(Style::default().fg(theme.accent)));
            f.render_widget(slot_list, left_chunks[1]);

            // Available items for selected slot with icons and colors
            let mut available_lines = Vec::new();

            for (i, item) in equipment_state.available_items.iter().enumerate() {
                let selected = i == equipment_state.selected_item_index;

                let (icon, type_color) = match &item.item_type {
                    crate::forge::ItemType::Weapon(_) => ("⚔", theme.primary),
                    crate::forge::ItemType::Armor(_) => ("🛡", theme.info),
                    crate::forge::ItemType::Accessory(_) => ("💍", Color::Magenta),
                    _ => ("📦", theme.text_secondary),
                };

                let line = if selected {
                    Line::from(vec![
                        Span::styled(format!("{} ", icon), Style::default().fg(type_color).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                        Span::styled(&item.name, Style::default().fg(theme.text_highlight).bg(theme.secondary).add_modifier(Modifier::BOLD)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(format!("{} ", icon), Style::default().fg(type_color)),
                        Span::styled(&item.name, Style::default().fg(theme.text_primary)),
                    ])
                };

                available_lines.push(ListItem::new(line));
            }

            if equipment_state.available_items.is_empty() {
                available_lines.push(ListItem::new(
                    Line::from(vec![
                        Span::styled("📭 ", Style::default().fg(theme.text_muted)),
                        Span::styled("No compatible items in inventory", Style::default().fg(theme.text_muted)),
                    ])
                ));
            }

            let (slot_name, slot_icon) = match equipment_state.selected_slot {
                EquipmentSlot::Weapon => ("Weapons", "⚔"),
                EquipmentSlot::Armor => ("Armor", "🛡"),
                EquipmentSlot::Shield => ("Shields", "🔰"),
                EquipmentSlot::Accessory1 | EquipmentSlot::Accessory2 => ("Accessories", "💍"),
            };

            let available_list = List::new(available_lines)
                .block(Block::default().borders(Borders::ALL).title(format!("{} Available {}", slot_icon, slot_name)).border_style(Style::default().fg(theme.info)));
            f.render_widget(available_list, main_chunks[1]);

            // Item details with enhanced visuals
            let details = if let Some(selected_item) = equipment_state.available_items.get(equipment_state.selected_item_index) {
                let divider_str = framework::art::divider(main_chunks[2].width.saturating_sub(4) as usize, framework::art::DividerStyle::Single);
                let mut detail_lines = vec![
                    Line::from(Span::styled(&selected_item.name, Style::default().fg(theme.text_highlight).add_modifier(Modifier::BOLD))),
                    Line::from(Span::styled(divider_str, Style::default().fg(theme.border_secondary))),
                    Line::from(""),
                    Line::from(Span::styled(selected_item.description.as_str(), Style::default().fg(theme.text_primary))),
                    Line::from(""),
                ];

                // Add item type-specific stats
                match &selected_item.item_type {
                    crate::forge::ItemType::Weapon(weapon_data) => {
                        detail_lines.push(Line::from(vec![
                            Span::styled("⚔ ", Style::default().fg(theme.primary)),
                            Span::styled("Weapon Stats", Style::default().fg(theme.text_secondary).add_modifier(Modifier::BOLD)),
                        ]));
                        detail_lines.push(Line::from(vec![
                            Span::styled("  Damage: ", Style::default().fg(theme.text_secondary)),
                            Span::styled(weapon_data.damage_dice.clone(), Style::default().fg(theme.error)),
                        ]));
                        detail_lines.push(Line::from(""));
                    },
                    crate::forge::ItemType::Armor(armor_data) => {
                        detail_lines.push(Line::from(vec![
                            Span::styled("🛡 ", Style::default().fg(theme.info)),
                            Span::styled("Armor Stats", Style::default().fg(theme.text_secondary).add_modifier(Modifier::BOLD)),
                        ]));
                        detail_lines.push(Line::from(vec![
                            Span::styled("  AR: ", Style::default().fg(theme.text_secondary)),
                            Span::styled(format!("+{}", armor_data.armor_rating), Style::default().fg(theme.success)),
                        ]));
                        detail_lines.push(Line::from(""));
                    },
                    _ => {},
                }

                detail_lines.push(Line::from(vec![
                    Span::styled("⚖ ", Style::default().fg(theme.text_secondary)),
                    Span::styled(format!("{:.1} kg", selected_item.weight), Style::default().fg(theme.text_primary)),
                    Span::styled("  🪙 ", Style::default().fg(theme.warning)),
                    Span::styled(format!("{} gp", selected_item.value), Style::default().fg(theme.warning)),
                ]));

                detail_lines
            } else {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("📋 ", Style::default().fg(theme.text_muted)),
                        Span::styled("Select an item to view details", Style::default().fg(theme.text_muted)),
                    ])
                ]
            };

            let details_panel = Paragraph::new(details)
                .block(Block::default().borders(Borders::ALL).title("📝 Item Details").border_style(Style::default().fg(Color::Magenta)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(details_panel, main_chunks[2]);

            // Controls with better styling
            let controls_content = vec![
                Line::from(vec![
                    Span::styled("↑↓/WS/JK ", Style::default().fg(theme.accent)),
                    Span::styled("Slots", Style::default().fg(theme.text_secondary)),
                    Span::styled(" • ", Style::default().fg(theme.text_muted)),
                    Span::styled("←→/AD/HL ", Style::default().fg(theme.accent)),
                    Span::styled("Items", Style::default().fg(theme.text_secondary)),
                    Span::styled(" • ", Style::default().fg(theme.text_muted)),
                    Span::styled("ENTER ", Style::default().fg(theme.success)),
                    Span::styled("Equip", Style::default().fg(theme.text_secondary)),
                ]),
                Line::from(vec![
                    Span::styled("U ", Style::default().fg(theme.warning)),
                    Span::styled("Unequip", Style::default().fg(theme.text_secondary)),
                    Span::styled(" • ", Style::default().fg(theme.text_muted)),
                    Span::styled("ESC ", Style::default().fg(theme.error)),
                    Span::styled("Back to Main Menu", Style::default().fg(theme.text_secondary)),
                ]),
            ];
            let controls = Paragraph::new(controls_content)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("⌨ Controls").border_style(Style::default().fg(theme.border_secondary)));
            f.render_widget(controls, left_chunks[2]);
            
        } else {
            let no_char = Paragraph::new("No character loaded.")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Equipment").border_style(Style::default().fg(Color::Red)));
            f.render_widget(no_char, area);
        }
    }

    // Character Sheet Panel Drawing Functions
    fn draw_basic_info_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let theme = framework::UITheme::forge_theme();

        let info_lines = vec![
            framework::art::section_header("CHARACTER", area.width.saturating_sub(4) as usize, &theme),
            Line::from(""),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.text_secondary)),
                Span::styled(&character.name, Style::default()
                    .fg(theme.text_highlight)
                    .add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Race: ", Style::default().fg(theme.text_secondary)),
                Span::styled(&character.race.name, Style::default().fg(theme.info)),
            ]),
            Line::from(vec![
                Span::styled("Level: ", Style::default().fg(theme.text_secondary)),
                Span::styled(character.level.to_string(), Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("XP: ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{}", character.experience), Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🪙 ", Style::default().fg(theme.warning)),
                Span::styled(format!("{} gold", character.gold), Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Created: ", Style::default().fg(theme.text_muted)),
                Span::styled(character.created_at.format("%Y-%m-%d").to_string(),
                    Style::default().fg(theme.text_secondary)),
            ]),
        ];

        let panel = Paragraph::new(info_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Basic Information")
                .border_style(Style::default().fg(theme.border_primary)))
            .style(Style::default().bg(theme.background));

        f.render_widget(panel, area);
    }

    fn draw_characteristics_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let theme = framework::UITheme::forge_theme();

        // Helper to color-code characteristics based on value
        let char_color = |val: f32| -> Color {
            if val >= 13.0 { theme.success }
            else if val >= 10.0 { theme.text_primary }
            else if val >= 7.0 { theme.warning }
            else { theme.error }
        };

        let char_lines = vec![
            framework::art::section_header("CHARACTERISTICS", area.width.saturating_sub(4) as usize, &theme),
            Line::from(""),
            Line::from(vec![
                Span::styled("STR ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4.1}", character.characteristics.strength),
                    Style::default().fg(char_color(character.characteristics.strength))),
                Span::styled("  Strength", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("STA ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4.1}", character.characteristics.stamina),
                    Style::default().fg(char_color(character.characteristics.stamina))),
                Span::styled("  Stamina", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("INT ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4.1}", character.characteristics.intellect),
                    Style::default().fg(char_color(character.characteristics.intellect))),
                Span::styled("  Intellect", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("INS ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4.1}", character.characteristics.insight),
                    Style::default().fg(char_color(character.characteristics.insight))),
                Span::styled("  Insight", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("DEX ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4.1}", character.characteristics.dexterity),
                    Style::default().fg(char_color(character.characteristics.dexterity))),
                Span::styled("  Dexterity", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("AWR ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4.1}", character.characteristics.awareness),
                    Style::default().fg(char_color(character.characteristics.awareness))),
                Span::styled("  Awareness", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("SPD ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4}", character.characteristics.speed),
                    Style::default().fg(theme.info)),
                Span::styled("  Speed", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("POW ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4}", character.characteristics.power),
                    Style::default().fg(theme.sp_color)),
                Span::styled("  Power", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("LUC ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:4}", character.characteristics.luck),
                    Style::default().fg(theme.warning)),
                Span::styled("  Luck", Style::default().fg(theme.text_muted)),
            ]),
        ];

        let panel = Paragraph::new(char_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Characteristics")
                .border_style(Style::default().fg(theme.border_primary)))
            .style(Style::default().bg(theme.background));

        f.render_widget(panel, area);
    }

    fn draw_race_abilities_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let mut race_lines = vec![
            Line::from(Span::styled("RACE & ABILITIES", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(format!("Race: {}", character.race.name)),
            Line::from(""),
            Line::from(character.race.description.clone()),
            Line::from(""),
            Line::from(Span::styled("Special Abilities:", Style::default().fg(Color::Magenta))),
        ];

        for ability in &character.race.special_abilities {
            race_lines.push(Line::from(format!("• {}", ability)));
        }

        let panel = Paragraph::new(race_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Race & Abilities")
                .border_style(Style::default().fg(Color::Magenta)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(panel, area);
    }

    fn draw_combat_stats_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let theme = framework::UITheme::forge_theme();

        let hp_current = character.combat_stats.hit_points.current;
        let hp_max = character.combat_stats.hit_points.max;
        let sp_current = character.magic.spell_points.current;
        let sp_max = character.magic.spell_points.max;

        let mut combat_lines = vec![
            framework::art::section_header("COMBAT", area.width.saturating_sub(4) as usize, &theme),
            Line::from(""),
            Line::from(vec![
                Span::styled("❤ HP: ", Style::default().fg(theme.error)),
                Span::styled(format!("{}/{}", hp_current, hp_max), Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD)),
            ]),
        ];

        // HP bar
        let hp_bar_width = area.width.saturating_sub(8).min(20) as usize;
        combat_lines.push(Line::from(
            framework::art::hp_bar_colored(hp_current, hp_max, hp_bar_width, &theme)
        ));

        combat_lines.push(Line::from(""));
        combat_lines.push(Line::from(vec![
            Span::styled("✨ SP: ", Style::default().fg(theme.sp_color)),
            Span::styled(format!("{}/{}", sp_current, sp_max), Style::default()
                .fg(theme.text_primary)
                .add_modifier(Modifier::BOLD)),
        ]));

        // SP bar
        let sp_bar = framework::art::progress_bar(sp_current, sp_max, hp_bar_width, '█', '░');
        combat_lines.push(Line::from(Span::styled(sp_bar, Style::default().fg(theme.sp_color))));

        combat_lines.push(Line::from(""));
        combat_lines.push(Line::from(vec![
            Span::styled("⚔ AV: ", Style::default().fg(theme.primary)),
            Span::styled(format!("{}", character.combat_stats.attack_value), Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)),
            Span::styled("  🛡 DV: ", Style::default().fg(theme.info)),
            Span::styled(format!("{}", character.combat_stats.defensive_value), Style::default()
                .fg(theme.info)
                .add_modifier(Modifier::BOLD)),
        ]));

        let dmg_color = if character.combat_stats.damage_bonus >= 0 { theme.success } else { theme.error };
        combat_lines.push(Line::from(vec![
            Span::styled("💥 DMG: ", Style::default().fg(theme.warning)),
            Span::styled(format!("{:+}", character.combat_stats.damage_bonus), Style::default()
                .fg(dmg_color)
                .add_modifier(Modifier::BOLD)),
        ]));

        let panel = Paragraph::new(combat_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Combat Statistics")
                .border_style(Style::default().fg(theme.border_primary)))
            .style(Style::default().bg(theme.background));

        f.render_widget(panel, area);
    }

    fn draw_derived_values_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        // Calculate derived values based on characteristics
        let endurance = (character.characteristics.strength + character.characteristics.stamina) / 2.0;
        let reflexes = (character.characteristics.dexterity + character.characteristics.awareness) / 2.0;
        let will = (character.characteristics.intellect + character.characteristics.insight) / 2.0;
        
        let derived_lines = vec![
            Line::from(Span::styled("DERIVED VALUES", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(format!("Endurance:  {:.1}", endurance)),
            Line::from(format!("Reflexes:   {:.1}", reflexes)),
            Line::from(format!("Will:       {:.1}", will)),
            Line::from(format!("Vision:     {}", character.vision_radius)),
        ];

        let panel = Paragraph::new(derived_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Derived Values")
                .border_style(Style::default().fg(Color::Blue)));
        
        f.render_widget(panel, area);
    }

    fn draw_character_skills_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let (basic_skills, percentile_skills, combat_skills, magic_skills) = character.get_skills_by_category();
        
        let mut skill_lines = vec![
            Line::from(Span::styled("SKILLS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
        ];

        // Basic Skills Section
        if !basic_skills.is_empty() {
            skill_lines.push(Line::from(Span::styled("Basic Skills:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            for (name, value, is_trained) in &basic_skills {
                let display_name = if *is_trained {
                    format!("✓ {}: {}%", name, value)
                } else {
                    format!("○ {}: {}% (untrained)", name, value)
                };
                let style = if *is_trained {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };
                skill_lines.push(Line::from(Span::styled(display_name, style)));
            }
            skill_lines.push(Line::from(""));
        }

        // Percentile Skills Section
        if !percentile_skills.is_empty() {
            skill_lines.push(Line::from(Span::styled("Percentile Skills:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
            for (name, value, is_trained) in &percentile_skills {
                let display_name = if *is_trained {
                    format!("✓ {}: {}%", name, value)
                } else {
                    format!("○ {}: {}% (untrained)", name, value)
                };
                let style = if *is_trained {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };
                skill_lines.push(Line::from(Span::styled(display_name, style)));
            }
            skill_lines.push(Line::from(""));
        }

        // Combat Skills Section
        if !combat_skills.is_empty() {
            skill_lines.push(Line::from(Span::styled("Combat Skills:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))));
            for (name, value, is_trained) in &combat_skills {
                let display_name = if *is_trained {
                    if let Some((level, percentage)) = character.combat_skills.get(name) {
                        if *level > 0 {
                            format!("✓ {}: {}% (Level {})", name, percentage, level)
                        } else {
                            format!("✓ {}: {}%", name, percentage)
                        }
                    } else {
                        format!("✓ {}: {}%", name, value)
                    }
                } else {
                    format!("○ {}: {}% (WSL 0, untrained)", name, value)
                };
                let style = if *is_trained {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };
                skill_lines.push(Line::from(Span::styled(display_name, style)));
            }
            skill_lines.push(Line::from(""));
        }

        // Magic Skills Section (brief, detailed info in magic panel)
        if !magic_skills.is_empty() {
            skill_lines.push(Line::from(Span::styled("Magic Skills:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))));
            for (name, value, is_trained) in &magic_skills {
                let display_name = if *is_trained {
                    format!("✓ {}: {}%", name, value)
                } else {
                    format!("○ {}: {}% (untrained)", name, value)
                };
                let style = if *is_trained {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };
                skill_lines.push(Line::from(Span::styled(display_name, style)));
            }
        }

        let panel = Paragraph::new(skill_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Skills (✓ Trained | ○ Untrained)")
                .border_style(Style::default().fg(Color::Green)));
        
        f.render_widget(panel, area);
    }

    fn draw_magic_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let mut magic_lines = vec![
            Line::from(Span::styled("MAGIC", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(format!("Spell Points: {}/{}", character.magic.spell_points.current, character.magic.spell_points.max)),
            Line::from(""),
        ];

        // Show magic schools
        if !character.magic.school_skills.is_empty() {
            magic_lines.push(Line::from(Span::styled("Magic Schools:", Style::default().fg(Color::Magenta))));
            for (school, level) in &character.magic.school_skills {
                magic_lines.push(Line::from(format!("{}: {}", school, level)));
            }
            magic_lines.push(Line::from(""));
        }

        // Show known spells
        if !character.magic.known_spells.is_empty() {
            magic_lines.push(Line::from(Span::styled("Known Spells:", Style::default().fg(Color::Cyan))));
            for (school, spells) in &character.magic.known_spells {
                magic_lines.push(Line::from(format!("{}:", school)));
                for spell in spells {
                    magic_lines.push(Line::from(format!("  • {}", spell)));
                }
            }
        } else {
            magic_lines.push(Line::from("No spells known"));
        }

        let panel = Paragraph::new(magic_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Magic")
                .border_style(Style::default().fg(Color::Magenta)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(panel, area);
    }

    fn draw_equipment_sheet_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let mut equipment_lines = vec![
            Line::from(Span::styled("EQUIPMENT", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
        ];

        // Weapon
        if let Some(weapon) = &character.equipment.weapon {
            equipment_lines.push(Line::from(format!("⚔️ Weapon: {}", weapon.name)));
            equipment_lines.push(Line::from(format!("   Damage: {} (+{})", weapon.damage_dice, weapon.damage_bonus)));
        } else {
            equipment_lines.push(Line::from("⚔️ Weapon: None"));
        }

        // Armor
        if let Some(armor) = &character.equipment.armor {
            equipment_lines.push(Line::from(format!("🛡️ Armor: {}", armor.name)));
            equipment_lines.push(Line::from(format!("   Rating: {}", armor.armor_rating)));
        } else {
            equipment_lines.push(Line::from("🛡️ Armor: None"));
        }

        // Shield
        if let Some(shield) = &character.equipment.shield {
            equipment_lines.push(Line::from(format!("🔰 Shield: {}", shield.name)));
        } else {
            equipment_lines.push(Line::from("🔰 Shield: None"));
        }

        // Accessories
        equipment_lines.push(Line::from(format!("💍 Accessory 1: {}", 
            character.equipment.accessory1.as_ref().map_or("None".to_string(), |a| a.name.clone()))));
        equipment_lines.push(Line::from(format!("💍 Accessory 2: {}", 
            character.equipment.accessory2.as_ref().map_or("None".to_string(), |a| a.name.clone()))));

        let panel = Paragraph::new(equipment_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Equipment")
                .border_style(Style::default().fg(Color::Blue)));
        
        f.render_widget(panel, area);
    }

    fn draw_movement_encumbrance_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let current_weight: f32 = character.inventory.items.iter()
            .map(|item| item.weight * item.quantity as f32)
            .sum();
        let weight_ratio = current_weight / character.inventory.max_weight;
        
        let encumbrance_status = if weight_ratio < 0.5 {
            "Light"
        } else if weight_ratio < 0.75 {
            "Medium"
        } else if weight_ratio < 1.0 {
            "Heavy"
        } else {
            "Overloaded"
        };

        let movement_lines = vec![
            Line::from(Span::styled("MOVEMENT", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(format!("Base Speed: {}", character.characteristics.speed)),
            Line::from(format!("Encumbrance: {}", encumbrance_status)),
            Line::from(format!("Weight: {:.1}/{:.1} kg", current_weight, character.inventory.max_weight)),
            Line::from(format!("Penalty: {:+.1}%", character.inventory.weight_penalty)),
        ];

        let panel = Paragraph::new(movement_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Movement & Encumbrance")
                .border_style(Style::default().fg(Color::Yellow)));
        
        f.render_widget(panel, area);
    }

    fn draw_inventory_sheet_panel(f: &mut Frame, area: ratatui::layout::Rect, character: &crate::forge::ForgeCharacter) {
        let mut inventory_lines = vec![
            Line::from(Span::styled("INVENTORY", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
        ];

        if character.inventory.items.is_empty() {
            inventory_lines.push(Line::from("No items"));
        } else {
            for item in &character.inventory.items {
                let quantity_text = if item.quantity > 1 {
                    format!(" ({})", item.quantity)
                } else {
                    String::new()
                };
                inventory_lines.push(Line::from(format!("• {}{}", item.name, quantity_text)));
                if item.weight > 0.0 {
                    inventory_lines.push(Line::from(format!("  {:.1} kg each", item.weight)));
                }
            }
        }

        let panel = Paragraph::new(inventory_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Inventory")
                .border_style(Style::default().fg(Color::Green)))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(panel, area);
    }
}

