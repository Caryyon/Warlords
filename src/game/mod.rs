use crate::forge::{ForgeCharacter, ForgeCharacterCreation, CombatParticipant, CombatAction, Weapon, Armor,
    create_wild_boar, create_wolf, create_goblin, create_bandit, create_orc, create_giant_spider, create_mountain_lion, create_skeleton, create_zombie};
use rand::Rng;
use crate::ui::{GameUI, UIState, CharacterCreationState, CreationStep, CombatState, WorldExplorationState, DungeonExplorationState, CombatPhase, mouse::ClickableArea};
use crate::world::{WorldManager, WorldCoord, LocalCoord};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, Event, MouseEvent};
use std::collections::HashMap;

// Centralized key binding system to prevent conflicts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameContext {
    WorldExploration,
    DungeonExploration,
    CharacterMenu,
    InventoryManagement,
    EquipmentManagement,
    Combat,
    CharacterCreation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameAction {
    // Movement
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    
    // UI Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    
    // Core Actions
    Confirm,
    Cancel,
    Inspect,
    
    // Inventory & Equipment
    OpenInventory,
    OpenEquipment,
    SortItems,
    FilterItems,
    DropItem,
    UseItem,
    EquipItem,
    UnequipItem,
    
    // Character & Menu
    OpenCharacterMenu,
    OpenCharacterSheet,
    OpenMap,
    Rest,
    Save,
    Quit,
    
    // Combat
    Attack,
    Defend,
    UseSkill,
    Flee,
    
    // Exploration
    Search,
    Enter,
    Climb,
    Light,
}

pub struct KeyBindings {
    bindings: HashMap<(GameContext, KeyCode), GameAction>,
    conflicts: Vec<(GameContext, KeyCode, Vec<GameAction>)>,
}

// Forge combat damage calculation result
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ForgeDamageResult {
    total_damage: u32,
    dice_count: u32,
    critical: bool,
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut kb = KeyBindings {
            bindings: HashMap::new(),
            conflicts: Vec::new(),
        };
        kb.setup_default_bindings();
        kb
    }
    
    fn bind(&mut self, context: GameContext, key: KeyCode, action: GameAction) {
        let key_binding = (context.clone(), key);
        
        // Check for conflicts within the same context
        if let Some(existing_action) = self.bindings.get(&key_binding) {
            if existing_action != &action {
                // Record conflict for later resolution
                let conflict_entry = self.conflicts.iter_mut()
                    .find(|(ctx, k, _)| ctx == &context && k == &key);
                    
                if let Some((_, _, actions)) = conflict_entry {
                    if !actions.contains(&action) {
                        actions.push(action.clone());
                    }
                } else {
                    self.conflicts.push((context.clone(), key, vec![existing_action.clone(), action.clone()]));
                }
            }
        }
        
        self.bindings.insert(key_binding, action);
    }
    
    pub fn get_action(&self, context: &GameContext, key: KeyCode) -> Option<&GameAction> {
        self.bindings.get(&(context.clone(), key))
    }
    
    pub fn check_conflicts(&self) -> &Vec<(GameContext, KeyCode, Vec<GameAction>)> {
        &self.conflicts
    }
    
    fn setup_default_bindings(&mut self) {
        use GameContext::*;
        use GameAction::*;
        
        // Movement keys (consistent across exploration contexts)
        // Supports: Arrow keys, WASD, and HJKL (vim)
        for context in [WorldExploration, DungeonExploration] {
            // North movement
            self.bind(context.clone(), KeyCode::Char('w'), MoveNorth);
            self.bind(context.clone(), KeyCode::Up, MoveNorth);
            self.bind(context.clone(), KeyCode::Char('k'), MoveNorth);
            // South movement
            self.bind(context.clone(), KeyCode::Char('s'), MoveSouth);
            self.bind(context.clone(), KeyCode::Down, MoveSouth);
            self.bind(context.clone(), KeyCode::Char('j'), MoveSouth);
            // West movement
            self.bind(context.clone(), KeyCode::Char('a'), MoveWest);
            self.bind(context.clone(), KeyCode::Left, MoveWest);
            self.bind(context.clone(), KeyCode::Char('h'), MoveWest);
            // East movement
            self.bind(context.clone(), KeyCode::Char('d'), MoveEast);
            self.bind(context.clone(), KeyCode::Right, MoveEast);
            self.bind(context.clone(), KeyCode::Char('l'), MoveEast);

            // Common exploration actions
            self.bind(context.clone(), KeyCode::Char('m'), OpenCharacterMenu);
            self.bind(context.clone(), KeyCode::Char('i'), Inspect);
            self.bind(context.clone(), KeyCode::Char('b'), OpenInventory); // 'b' for bag
            self.bind(context.clone(), KeyCode::Enter, Enter);
            self.bind(context.clone(), KeyCode::Char(' '), Search);
            self.bind(context.clone(), KeyCode::Char('r'), Rest);
            self.bind(context.clone(), KeyCode::Char('L'), Light);  // Capital L to avoid conflict with 'l' movement
            self.bind(context, KeyCode::Esc, Cancel);
        }
        
        // Character menu
        self.bind(CharacterMenu, KeyCode::Char('i'), OpenInventory);
        self.bind(CharacterMenu, KeyCode::Char('e'), OpenEquipment);
        self.bind(CharacterMenu, KeyCode::Char('c'), OpenCharacterSheet);
        self.bind(CharacterMenu, KeyCode::Char('m'), Cancel);
        self.bind(CharacterMenu, KeyCode::Esc, Cancel);
        self.bind(CharacterMenu, KeyCode::Char('q'), Quit);
        
        // Inventory management (supports: Arrow keys, WASD, HJKL)
        self.bind(InventoryManagement, KeyCode::Up, NavigateUp);
        self.bind(InventoryManagement, KeyCode::Char('w'), NavigateUp);
        self.bind(InventoryManagement, KeyCode::Char('k'), NavigateUp);
        self.bind(InventoryManagement, KeyCode::Down, NavigateDown);
        self.bind(InventoryManagement, KeyCode::Char('s'), NavigateDown);
        self.bind(InventoryManagement, KeyCode::Char('j'), NavigateDown);
        self.bind(InventoryManagement, KeyCode::Enter, UseItem);
        self.bind(InventoryManagement, KeyCode::Char('d'), DropItem);
        self.bind(InventoryManagement, KeyCode::Tab, SortItems);
        self.bind(InventoryManagement, KeyCode::Char('f'), FilterItems);
        self.bind(InventoryManagement, KeyCode::Esc, Cancel);
        
        // Equipment management (supports: Arrow keys, WASD, HJKL)
        self.bind(EquipmentManagement, KeyCode::Up, NavigateUp);
        self.bind(EquipmentManagement, KeyCode::Char('w'), NavigateUp);
        self.bind(EquipmentManagement, KeyCode::Char('k'), NavigateUp);
        self.bind(EquipmentManagement, KeyCode::Down, NavigateDown);
        self.bind(EquipmentManagement, KeyCode::Char('s'), NavigateDown);
        self.bind(EquipmentManagement, KeyCode::Char('j'), NavigateDown);
        self.bind(EquipmentManagement, KeyCode::Left, NavigateLeft);
        self.bind(EquipmentManagement, KeyCode::Char('a'), NavigateLeft);
        self.bind(EquipmentManagement, KeyCode::Char('h'), NavigateLeft);
        self.bind(EquipmentManagement, KeyCode::Right, NavigateRight);
        self.bind(EquipmentManagement, KeyCode::Char('d'), NavigateRight);
        self.bind(EquipmentManagement, KeyCode::Char('l'), NavigateRight);
        self.bind(EquipmentManagement, KeyCode::Enter, EquipItem);
        self.bind(EquipmentManagement, KeyCode::Char('u'), UnequipItem);
        self.bind(EquipmentManagement, KeyCode::Esc, Cancel);
        
        // Combat (action keys: a/d/s/f, navigation: arrows/jk/hl)
        // Note: WASD reserved for combat actions, not navigation
        self.bind(Combat, KeyCode::Char('a'), Attack);
        self.bind(Combat, KeyCode::Char('d'), Defend);
        self.bind(Combat, KeyCode::Char('s'), UseSkill);
        self.bind(Combat, KeyCode::Char('f'), Flee);
        self.bind(Combat, KeyCode::Up, NavigateUp);
        self.bind(Combat, KeyCode::Char('k'), NavigateUp);
        self.bind(Combat, KeyCode::Down, NavigateDown);
        self.bind(Combat, KeyCode::Char('j'), NavigateDown);
        self.bind(Combat, KeyCode::Left, NavigateLeft);
        self.bind(Combat, KeyCode::Char('h'), NavigateLeft);
        self.bind(Combat, KeyCode::Right, NavigateRight);
        self.bind(Combat, KeyCode::Char('l'), NavigateRight);
        self.bind(Combat, KeyCode::Enter, Confirm);
        self.bind(Combat, KeyCode::Esc, Cancel);
    }
    
    pub fn print_bindings(&self, context: &GameContext) {
        println!("=== Key Bindings for {:?} ===", context);
        let mut context_bindings: Vec<_> = self.bindings.iter()
            .filter(|((ctx, _), _)| ctx == context)
            .collect();
        context_bindings.sort_by_key(|((_, key), _)| format!("{:?}", key));
        
        for ((_, key), action) in context_bindings {
            println!("{:?} -> {:?}", key, action);
        }
    }
    
    pub fn print_all_conflicts(&self) {
        if self.conflicts.is_empty() {
            println!("✅ No key binding conflicts detected!");
            return;
        }
        
        println!("⚠️  Key Binding Conflicts Detected:");
        for (context, key, actions) in &self.conflicts {
            println!("  {:?} + {:?} -> {:?}", context, key, actions);
        }
    }
}

pub struct Game {
    ui: GameUI,
    state: UIState,
    state_manager: crate::state::GameStateManager,
    current_character: Option<ForgeCharacter>,
    input_buffer: String,
    world_manager: Option<WorldManager>,
    player_position: WorldCoord,
    saved_world_state: Option<WorldExplorationState>,
    #[allow(dead_code)]
    key_bindings: KeyBindings,
    mouse_handler: crate::ui::mouse::MouseHandler,
}

impl Game {
    pub fn new() -> anyhow::Result<Self> {
        let ui = GameUI::new()?;
        let state_manager = crate::state::GameStateManager::new(".")?;
        let key_bindings = KeyBindings::new();

        // Check for key binding conflicts at startup
        key_bindings.print_all_conflicts();

        Ok(Game {
            ui,
            state: UIState::Welcome,
            state_manager,
            current_character: None,
            input_buffer: String::new(),
            world_manager: None,
            player_position: WorldCoord::new(256, 256), // Start in center of world
            saved_world_state: None,
            key_bindings,
            mouse_handler: crate::ui::mouse::MouseHandler::new(),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.ui.draw(&self.state, &self.input_buffer, self.current_character.as_ref())?;

            if let Some(event) = self.ui.handle_input()? {
                match event {
                    Event::Key(key) => {
                        if self.handle_key_event(key)? {
                            break; // Exit game
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse)?;
                    }
                    _ => {}
                }
            }
        }

        // Graceful shutdown
        self.shutdown()?;
        Ok(())
    }
    
    fn shutdown(&mut self) -> anyhow::Result<()> {
        // Save world data if it exists
        if let Some(world_manager) = &mut self.world_manager {
            world_manager.save_if_dirty()?;
        }

        // Save character data
        if let Some(character) = &mut self.current_character {
            character.update_last_played();
            self.state_manager.update_character(character)?;
        }

        // Cleanup UI
        self.ui.cleanup()?;

        println!("Game saved and exited gracefully. Thank you for playing Warlords!");
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        // Handle Ctrl+C globally for graceful shutdown
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            return Ok(true); // Exit game
        }

        match &self.state {
            UIState::Welcome => {
                // Any key proceeds to main menu
                self.state = UIState::MainMenu;
            }
            UIState::MainMenu => {
                // Main menu - different options based on whether character is logged in
                if self.current_character.is_some() {
                    // Logged in: Options 1-5 (Return to Game, Explore, Character Menu, Logout, Quit)
                    match key.code {
                        KeyCode::Char('1') => {
                            self.state = UIState::Playing;
                        }
                        KeyCode::Char('2') => {
                            // Enter world exploration directly
                            self.enter_world_exploration()?;
                        }
                        KeyCode::Char('3') => {
                            self.state = UIState::CharacterMenu;
                        }
                        KeyCode::Char('4') => {
                            // Logout and return to character list
                            // Save character data before logging out
                            if let Some(character) = &self.current_character {
                                let _ = self.state_manager.update_character(character);
                            }
                            self.current_character = None;
                            self.world_manager = None;
                            let character_list = self.state_manager.list_characters()?;
                            let selected_index = if character_list.is_empty() { None } else { Some(0) };
                            self.state = UIState::CharacterList(character_list, selected_index);
                        }
                        KeyCode::Char('5') | KeyCode::Char('q') => {
                            // Save character data before exiting
                            if let Some(character) = &self.current_character {
                                let _ = self.state_manager.update_character(character);
                            }
                            return Ok(true); // Exit
                        }
                        KeyCode::Char('m') => {
                            // Quick return to game
                            self.state = UIState::Playing;
                        }
                        _ => {}
                    }
                } else {
                    // Not logged in: Options 1-4 (Login, Create, List Characters, Quit)
                    match key.code {
                        KeyCode::Char('1') => {
                            // Login to Existing Character -> go to character list
                            let character_list = self.state_manager.list_characters()?;
                            let selected_index = if character_list.is_empty() { None } else { Some(0) };
                            self.state = UIState::CharacterList(character_list, selected_index);
                        }
                        KeyCode::Char('2') => {
                            // Create New Character -> start character creation
                            self.state = UIState::CharacterCreation(crate::ui::CharacterCreationState {
                                step: crate::ui::CreationStep::Rolling,
                                rolled_data: None,
                                selected_race: None,
                                character_name: None,
                                selected_skills: Vec::new(),
                                available_skill_points: 0,
                                selected_spells: Vec::new(),
                                available_spell_picks: 0,
                                selected_gear: Vec::new(),
                                current_selection_index: 0,
                                available_skills_list: Vec::new(),
                                available_spells_list: Vec::new(),
                                available_gear_list: Vec::new(),
                                starting_gold: 100,
                                spent_gold: 0,
                            });
                        }
                        KeyCode::Char('3') => {
                            // List Characters -> go to character list
                            let character_list = self.state_manager.list_characters()?;
                            let selected_index = if character_list.is_empty() { None } else { Some(0) };
                            self.state = UIState::CharacterList(character_list, selected_index);
                        }
                        KeyCode::Char('4') | KeyCode::Char('q') => {
                            // Quit
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
            }
            UIState::CharacterCreation(creation_state) => {
                self.handle_character_creation_input(key, creation_state.clone())?;
            }
            UIState::CharacterList(character_list, selected_index) => {
                self.handle_character_list_input(key, character_list.clone(), *selected_index)?;
            }
            UIState::Playing => {
                match key.code {
                    KeyCode::Char('m') => {
                        self.state = UIState::MainMenu;
                    }
                    KeyCode::Char('q') => {
                        return Ok(true); // Exit
                    }
                    KeyCode::Char('e') => {
                        // Enter world exploration
                        if self.current_character.is_some() {
                            self.enter_world_exploration()?;
                        }
                    }
                    KeyCode::Char('c') => {
                        // Open character menu
                        if self.current_character.is_some() {
                            self.state = UIState::CharacterMenu;
                        }
                    }
                    KeyCode::Char('f') => {
                        // Start a test combat encounter
                        if self.current_character.is_some() {
                            let character = self.current_character.as_ref().unwrap().clone();
                            self.start_combat_encounter(&character)?;
                        }
                    }
                    // Add movement and game commands here
                    _ => {}
                }
            }
            UIState::WorldExploration(world_state) => {
                if self.handle_world_exploration_input(key, world_state.clone())? {
                    return Ok(true); // Exit game
                }
            }
            UIState::DungeonExploration(dungeon_state) => {
                if self.handle_dungeon_exploration_input(key, dungeon_state.clone())? {
                    return Ok(true); // Exit game
                }
            }
            UIState::CharacterMenu => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('m') => {
                        self.state = UIState::Playing;
                    }
                    KeyCode::Char('i') => {
                        // Enter inventory management
                        let inventory_state = crate::ui::InventoryState {
                            selected_index: 0,
                            scroll_offset: 0,
                            view_mode: crate::ui::InventoryViewMode::List,
                            sort_mode: crate::ui::InventorySortMode::Name,
                            filter_type: None,
                            showing_details: false,
                            selected_item_details: None,
                            sorted_indices: Vec::new(),
                        };
                        self.state = UIState::InventoryManagement(inventory_state);
                    }
                    KeyCode::Char('e') => {
                        // Enter equipment management
                        let equipment_state = crate::ui::EquipmentState {
                            selected_slot: crate::ui::EquipmentSlot::Weapon,
                            showing_details: false,
                            available_items: Vec::new(), // Will be populated based on selected slot
                            selected_item_index: 0,
                        };
                        self.state = UIState::EquipmentManagement(equipment_state);
                    }
                    KeyCode::Char('c') => {
                        // Open character sheet
                        self.state = UIState::CharacterSheet;
                    }
                    KeyCode::Char('q') => {
                        return Ok(true); // Exit
                    }
                    _ => {}
                }
            }
            UIState::InventoryManagement(inventory_state) => {
                self.handle_inventory_input(key, inventory_state.clone())?;
            }
            UIState::EquipmentManagement(equipment_state) => {
                self.handle_equipment_input(key, equipment_state.clone())?;
            }
            UIState::CharacterSheet => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('c') => {
                        self.state = UIState::CharacterMenu;
                    }
                    _ => {}
                }
            }
            UIState::Combat(_) => {
                // Legacy combat state - should not be used anymore
                // All combat now uses TacticalCombat
                return Err(anyhow::anyhow!("Legacy combat state encountered - this should not happen"));
            }
            UIState::TacticalCombat(tactical_state) => {
                self.handle_tactical_combat_input(key, tactical_state.clone())?;
            }
        }
        Ok(false)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> anyhow::Result<()> {
        use crate::ui::mouse::MouseInteraction;
        
        // Get clickable areas based on current UI state
        let clickable_areas = self.get_clickable_areas_for_state();
        
        // Process mouse event
        let interaction = self.mouse_handler.handle_mouse_event(mouse, &clickable_areas);
        
        match interaction {
            MouseInteraction::ButtonClicked(button_id) => {
                self.handle_button_click(&button_id)?;
            }
            MouseInteraction::ListItemSelected(index) => {
                self.handle_list_selection(index)?;
            }
            MouseInteraction::AreaClicked(area_id, x, y) => {
                self.handle_area_click(&area_id, x, y)?;
            }
            MouseInteraction::Scroll(direction, x, y) => {
                self.handle_scroll(direction, x, y)?;
            }
            MouseInteraction::Drag(area_id, start, end) => {
                self.handle_drag(&area_id, start, end)?;
            }
            MouseInteraction::None => {
                // No interaction, just update hover state
            }
        }
        
        Ok(())
    }
    
    fn get_clickable_areas_for_state(&self) -> Vec<ClickableArea> {
        use crate::ui::mouse::ClickableArea;
        use ratatui::layout::Rect;
        
        let mut areas = Vec::new();
        
        match &self.state {
            UIState::MainMenu => {
                // Add clickable areas for main menu buttons
                areas.push(ClickableArea::button("login".to_string(), Rect::new(0, 10, 20, 1)));
                areas.push(ClickableArea::button("create".to_string(), Rect::new(0, 12, 20, 1)));
                areas.push(ClickableArea::button("load".to_string(), Rect::new(0, 14, 20, 1)));
                areas.push(ClickableArea::button("quit".to_string(), Rect::new(0, 16, 20, 1)));
            }
            UIState::CharacterList(characters, _) => {
                // Add clickable areas for character list items
                for (i, _) in characters.iter().enumerate() {
                    let y = 5 + i as u16 * 2;
                    areas.push(ClickableArea::list_item(
                        format!("character_{}", i),
                        Rect::new(5, y, 50, 1),
                        i
                    ));
                }
            }
            UIState::WorldExploration(_) => {
                // Add world map as clickable area
                areas.push(ClickableArea::battlefield(
                    "worldmap".to_string(),
                    Rect::new(0, 0, 80, 40) // Most of the screen
                ));
            }
            _ => {
                // For other states, minimal clickable areas
            }
        }
        
        areas
    }
    
    fn handle_button_click(&mut self, button_id: &str) -> anyhow::Result<()> {
        match button_id {
            "create" => {
                self.state = UIState::CharacterCreation(crate::ui::CharacterCreationState {
                    step: crate::ui::CreationStep::Rolling,
                    rolled_data: None,
                    selected_race: None,
                    character_name: None,
                    selected_skills: Vec::new(),
                    available_skill_points: 0,
                    selected_spells: Vec::new(),
                    available_spell_picks: 0,
                    selected_gear: Vec::new(),
                    current_selection_index: 0,
                    available_skills_list: Vec::new(),
                    available_spells_list: Vec::new(),
                    available_gear_list: Vec::new(),
                    starting_gold: 100,
                    spent_gold: 0,
                });
            }
            "load" => {
                let character_list = self.state_manager.list_characters()?;
                let selected_index = if character_list.is_empty() { None } else { Some(0) };
                self.state = UIState::CharacterList(character_list, selected_index);
            }
            "quit" => {
                // This should trigger game exit
            }
            "spell" => {
                // Handle spell casting in tactical combat
                // TODO: Implement spell menu in new tactical combat system
                // if let UIState::TacticalCombat(ref mut tactical_state) = &mut self.state {
                //     tactical_state.spell_menu_open = true;
                // }
            }
            "close" => {
                // Handle modal/popup close
                // TODO: Implement popup menus in new tactical combat system
                // if let UIState::TacticalCombat(ref mut tactical_state) = &mut self.state {
                //     tactical_state.spell_menu_open = false;
                //     tactical_state.enhancement_menu_open = false;
                // }
            }
            _ => {
                // Unknown button, ignore
            }
        }
        Ok(())
    }
    
    fn handle_list_selection(&mut self, index: usize) -> anyhow::Result<()> {
        match &mut self.state {
            UIState::CharacterList(_, ref mut selected_index) => {
                *selected_index = Some(index);
                // Could also implement double-click to load character
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_area_click(&mut self, area_id: &str, _x: u16, _y: u16) -> anyhow::Result<()> {
        match area_id {
            "worldmap" => {
                // Handle world map clicks for movement
                if let UIState::WorldExploration(_) = &self.state {
                    // Could implement click-to-move on world map
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_scroll(&mut self, _direction: crate::ui::mouse::ScrollDirection, _x: u16, _y: u16) -> anyhow::Result<()> {
        // Handle scrolling for lists, menus, etc.
        // Implementation depends on what's being scrolled
        Ok(())
    }
    
    fn handle_drag(&mut self, _area_id: &str, _start: (u16, u16), _end: (u16, u16)) -> anyhow::Result<()> {
        // Handle drag operations (could be used for drag-to-move, etc.)
        Ok(())
    }

    fn handle_character_creation_input(&mut self, key: KeyEvent, mut creation_state: CharacterCreationState) -> anyhow::Result<()> {
        match creation_state.step {
            CreationStep::Rolling => {
                match key.code {
                    KeyCode::Enter | KeyCode::Char('r') => {
                        // Roll characteristics
                        let rolled_data = ForgeCharacterCreation::roll_characteristics();
                        creation_state.rolled_data = Some(rolled_data);
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Char('c') => {
                        if creation_state.rolled_data.is_some() {
                            // Continue to race selection
                            creation_state.step = CreationStep::RaceSelection;
                            self.state = UIState::CharacterCreation(creation_state);
                        }
                    }
                    KeyCode::Esc => {
                        self.state = UIState::MainMenu;
                    }
                    _ => {}
                }
            }
            CreationStep::RaceSelection => {
                match key.code {
                    KeyCode::Char(c) => {
                        let races = ForgeCharacterCreation::get_available_races();
                        let race_index = match c {
                            '1'..='9' => Some(c.to_digit(10).unwrap() as usize - 1),
                            '0' => Some(9), // Merikii is at index 9
                            '#' => Some(10), // Sprite is at index 10
                            _ => None,
                        };
                        
                        if let Some(idx) = race_index {
                            if idx < races.len() {
                                creation_state.selected_race = Some(races[idx].clone());
                                creation_state.step = CreationStep::NameEntry;
                                self.state = UIState::CharacterCreation(creation_state);
                            }
                        }
                    }
                    KeyCode::Esc => {
                        creation_state.step = CreationStep::Rolling;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    _ => {}
                }
            }
            CreationStep::NameEntry => {
                match key.code {
                    KeyCode::Enter => {
                        if self.input_buffer.len() >= 2 {
                            creation_state.character_name = Some(self.input_buffer.clone());
                            // Calculate available skill points based on race and characteristics
                            creation_state.available_skill_points = self.calculate_skill_points(&creation_state);
                            creation_state.available_skills_list = self.get_available_skills(&creation_state);
                            creation_state.current_selection_index = 0;
                            creation_state.step = CreationStep::SkillSelection;
                            self.state = UIState::CharacterCreation(creation_state);
                            self.input_buffer.clear();
                        }
                    }
                    KeyCode::Char(c) => {
                        self.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                    }
                    KeyCode::Esc => {
                        creation_state.step = CreationStep::RaceSelection;
                        self.state = UIState::CharacterCreation(creation_state);
                        self.input_buffer.clear();
                    }
                    _ => {}
                }
            }
            CreationStep::SkillSelection => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        if creation_state.current_selection_index > 0 {
                            creation_state.current_selection_index -= 1;
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        if creation_state.current_selection_index < creation_state.available_skills_list.len().saturating_sub(1) {
                            creation_state.current_selection_index += 1;
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Enter => {
                        // Select/deselect skill
                        if creation_state.current_selection_index < creation_state.available_skills_list.len() {
                            let skill = creation_state.available_skills_list[creation_state.current_selection_index].clone();
                            if creation_state.selected_skills.contains(&skill) {
                                // Deselect skill
                                creation_state.selected_skills.retain(|s| s != &skill);
                                creation_state.available_skill_points += 1;
                            } else if creation_state.available_skill_points > 0 {
                                // Select skill
                                creation_state.selected_skills.push(skill);
                                creation_state.available_skill_points -= 1;
                            }
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Char('c') => {
                        // Continue to spell selection
                        creation_state.available_spell_picks = self.calculate_spell_picks(&creation_state);
                        creation_state.available_spells_list = self.get_available_spells(&creation_state);
                        creation_state.current_selection_index = 0;
                        creation_state.step = CreationStep::SpellSelection;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Esc => {
                        creation_state.step = CreationStep::NameEntry;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    _ => {}
                }
            }
            CreationStep::SpellSelection => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        if creation_state.current_selection_index > 0 {
                            creation_state.current_selection_index -= 1;
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        if creation_state.current_selection_index < creation_state.available_spells_list.len().saturating_sub(1) {
                            creation_state.current_selection_index += 1;
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Enter => {
                        // Select/deselect spell
                        if creation_state.current_selection_index < creation_state.available_spells_list.len() {
                            let spell = creation_state.available_spells_list[creation_state.current_selection_index].clone();
                            if creation_state.selected_spells.contains(&spell) {
                                // Deselect spell
                                creation_state.selected_spells.retain(|s| s != &spell);
                                creation_state.available_spell_picks += 1;
                            } else if creation_state.available_spell_picks > 0 {
                                // Select spell
                                creation_state.selected_spells.push(spell);
                                creation_state.available_spell_picks -= 1;
                            }
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Char('c') => {
                        // Continue to gear selection
                        creation_state.available_gear_list = self.get_available_gear(&creation_state);
                        creation_state.current_selection_index = 0;
                        creation_state.step = CreationStep::GearSelection;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Esc => {
                        creation_state.step = CreationStep::SkillSelection;
                        creation_state.current_selection_index = 0;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    _ => {}
                }
            }
            CreationStep::GearSelection => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        if creation_state.current_selection_index > 0 {
                            creation_state.current_selection_index -= 1;
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        if creation_state.current_selection_index < creation_state.available_gear_list.len().saturating_sub(1) {
                            creation_state.current_selection_index += 1;
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Enter => {
                        // Select/deselect gear
                        if creation_state.current_selection_index < creation_state.available_gear_list.len() {
                            let (gear_name, cost) = creation_state.available_gear_list[creation_state.current_selection_index].clone();
                            if creation_state.selected_gear.contains(&gear_name) {
                                // Deselect gear - refund the gold
                                creation_state.selected_gear.retain(|g| g != &gear_name);
                                creation_state.spent_gold -= cost;
                            } else {
                                // Select gear if we can afford it
                                if creation_state.spent_gold + cost <= creation_state.starting_gold {
                                    creation_state.selected_gear.push(gear_name);
                                    creation_state.spent_gold += cost;
                                }
                            }
                        }
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Char('c') => {
                        // Continue to confirmation
                        creation_state.step = CreationStep::Confirmation;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    KeyCode::Esc => {
                        creation_state.step = CreationStep::SpellSelection;
                        creation_state.current_selection_index = 0;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    _ => {}
                }
            }
            CreationStep::Confirmation => {
                match key.code {
                    KeyCode::Enter => {
                        // Finalize character creation
                        if let (Some(rolled_data), Some(race), Some(name)) = (
                            &creation_state.rolled_data,
                            &creation_state.selected_race,
                            &creation_state.character_name,
                        ) {
                            let characteristics = ForgeCharacterCreation::apply_racial_modifiers(rolled_data, race);
                            let mut character = ForgeCharacterCreation::create_character(
                                name.clone(),
                                characteristics,
                                race.clone(),
                            );
                            
                            // Apply selected skills, spells, and gear
                            self.apply_character_selections(&mut character, &creation_state);

                            match self.state_manager.create_character(character.clone()) {
                                Ok(()) => {
                                    // Return to character list after creation
                                    let character_list = self.state_manager.list_characters()?;
                                    let selected_index = if character_list.is_empty() { None } else { Some(0) };
                                    self.state = UIState::CharacterList(character_list, selected_index);
                                }
                                Err(_) => {
                                    // Show error - character already exists
                                    // Return to character list
                                    let character_list = self.state_manager.list_characters()?;
                                    let selected_index = if character_list.is_empty() { None } else { Some(0) };
                                    self.state = UIState::CharacterList(character_list, selected_index);
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        creation_state.step = CreationStep::GearSelection;
                        self.state = UIState::CharacterCreation(creation_state);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn calculate_skill_points(&self, creation_state: &CharacterCreationState) -> u8 {
        // Base skill points = 3, plus bonus from race or high intellect
        let mut points = 3;
        
        if let Some(rolled_data) = &creation_state.rolled_data {
            // Bonus for high intellect
            if rolled_data.intellect.total >= 12.0 {
                points += 1;
            }
        }
        
        if let Some(race) = &creation_state.selected_race {
            // Some races get bonus skill points
            match race.name.as_str() {
                "Human" => points += 1, // Humans are versatile
                "Elf" => points += 1,   // Elves are learned
                _ => {}
            }
        }
        
        points
    }
    
    fn calculate_spell_picks(&self, creation_state: &CharacterCreationState) -> u8 {
        // Base 1 spell pick, plus bonus for magical races or high power
        let mut picks = 1;
        
        if let Some(rolled_data) = &creation_state.rolled_data {
            // Bonus for high power
            if rolled_data.power.total >= 15.0 {
                picks += 1;
            }
        }
        
        if let Some(race) = &creation_state.selected_race {
            // Magical races get bonus spells
            match race.name.as_str() {
                "Elf" | "Dunnar" | "Kithsara" => picks += 1,
                "Berserker" => picks = 0, // Berserkers fear magic
                _ => {}
            }
        }
        
        picks
    }
    
    fn get_available_skills(&self, creation_state: &CharacterCreationState) -> Vec<String> {
        let mut skills = vec![
            // Forge Weapon Combat Skills (Roll <= skill % to advance)
            "Sword".to_string(),
            "Axe".to_string(),
            "Mace".to_string(),
            "Spear".to_string(),
            "Dagger".to_string(),
            "Bow".to_string(),
            "Crossbow".to_string(),
            "Javelin".to_string(),
            "Sling".to_string(),
            "Staff".to_string(),
            "Shield".to_string(),
            "Unarmed Combat".to_string(),
            
            // Forge General Skills (Roll >= skill % to advance)
            "Athletics".to_string(),
            "Climb".to_string(),
            "Dodge".to_string(),
            "Hide".to_string(),
            "Listen".to_string(),
            "Move Silently".to_string(),
            "Open Locks".to_string(),
            "Pick Pockets".to_string(),
            "Spot".to_string(),
            "Tracking".to_string(),
            "Traps".to_string(),
            "Swim".to_string(),
            "Jump".to_string(),
            "Ride".to_string(),
            
            // Knowledge & Social Skills
            "Animal Handling".to_string(),
            "Craft".to_string(),
            "Healing".to_string(),
            "Herbalism".to_string(),
            "Lore".to_string(),
            "Language".to_string(),
            "Persuasion".to_string(),
            "Intimidation".to_string(),
            "Leadership".to_string(),
            "Survival".to_string(),
            "Seamanship".to_string(),
            
            // Magic School Skills (handled separately by magic system)
            "Beast Magic".to_string(),
            "Elemental Magic".to_string(),
            "Enchantment Magic".to_string(),
            "Necromancer Magic".to_string(),
            "Divine Magic".to_string(),
            "Mind Magic".to_string(),
        ];
        
        // Add race-specific skills that aren't in the base list
        if let Some(race) = &creation_state.selected_race {
            match race.name.as_str() {
                "Dwarf" => {
                    skills.push("Smithing".to_string());
                    skills.push("Mining".to_string());
                    skills.push("Stone Working".to_string());
                }
                "Elf" => {
                    skills.push("Nature Lore".to_string());
                    skills.push("Elven Blade Dancing".to_string());
                }
                "Berserker" => {
                    skills.push("Berserker Rage".to_string());
                    skills.push("Battle Fury".to_string());
                }
                "Higmoni" => {
                    skills.push("Desert Survival".to_string());
                    skills.push("Heat Resistance".to_string());
                }
                "Jher-em" => {
                    skills.push("Telepathy".to_string());
                    skills.push("Mental Contact".to_string());
                }
                "Kithsara" => {
                    skills.push("Nature Magic".to_string());
                    skills.push("Plant Lore".to_string());
                }
                "Merikii" => {
                    skills.push("Beast Speech".to_string());
                    skills.push("Animal Empathy".to_string());
                }
                "Sprite" => {
                    skills.push("Flight".to_string());
                    skills.push("Size Change".to_string());
                }
                "Ghantu" => {
                    skills.push("Brawling".to_string());
                    skills.push("Thick Skin".to_string());
                }
                _ => {}
            }
        }
        
        skills.sort();
        skills
    }
    
    fn get_available_spells(&self, creation_state: &CharacterCreationState) -> Vec<(String, crate::forge::magic::MagicSchool)> {
        use crate::forge::magic::MagicSchool;
        
        let mut spells = Vec::new();
        
        // Only show spells from magic schools the player has as skills
        if creation_state.selected_skills.contains(&"Beast Magic".to_string()) {
            spells.push(("Animal Communication".to_string(), MagicSchool::Beast));
            spells.push(("Bear Strength".to_string(), MagicSchool::Beast));
        }
        
        if creation_state.selected_skills.contains(&"Elemental Magic".to_string()) {
            spells.push(("Fire Bolt".to_string(), MagicSchool::Elemental));
            spells.push(("Lightning Strike".to_string(), MagicSchool::Elemental));
        }
        
        if creation_state.selected_skills.contains(&"Enchantment Magic".to_string()) {
            spells.push(("Weapon Blessing".to_string(), MagicSchool::Enchantment));
            spells.push(("Shield of Faith".to_string(), MagicSchool::Enchantment));
        }
        
        if creation_state.selected_skills.contains(&"Necromancer Magic".to_string()) {
            spells.push(("Drain Life".to_string(), MagicSchool::Necromancer));
            spells.push(("Weaken".to_string(), MagicSchool::Necromancer));
        }
        
        if creation_state.selected_skills.contains(&"Divine Magic".to_string()) {
            spells.push(("Heal Wounds".to_string(), MagicSchool::Divine));
            spells.push(("Turn Undead".to_string(), MagicSchool::Divine));
        }
        
        // Filter based on race restrictions
        if let Some(race) = &creation_state.selected_race {
            if race.name == "Berserker" {
                // Berserkers can't use magic
                spells.clear();
            }
        }
        
        spells
    }
    
    fn get_available_gear(&self, creation_state: &CharacterCreationState) -> Vec<(String, u32)> {
        let mut gear = vec![
            // Weapons
            ("Dagger".to_string(), 2),
            ("Short Sword".to_string(), 10),
            ("Long Sword".to_string(), 15),
            ("Hand Axe".to_string(), 5),
            ("Battle Axe".to_string(), 20),
            ("War Hammer".to_string(), 25),
            ("Spear".to_string(), 5),
            ("Short Bow".to_string(), 25),
            ("Crossbow".to_string(), 35),
            ("Staff".to_string(), 5),
            
            // Armor
            ("Leather Armor".to_string(), 10),
            ("Studded Leather".to_string(), 25),
            ("Chain Mail".to_string(), 75),
            ("Scale Mail".to_string(), 50),
            ("Plate Mail".to_string(), 400), // Expensive!
            ("Small Shield".to_string(), 10),
            ("Medium Shield".to_string(), 15),
            ("Large Shield".to_string(), 20),
            
            // Adventuring Gear
            ("Backpack".to_string(), 2),
            ("Rope (50 ft)".to_string(), 1),
            ("Torch (5)".to_string(), 1),
            ("Rations (1 week)".to_string(), 5),
            ("Waterskin".to_string(), 1),
            ("Bedroll".to_string(), 2),
            ("Thieves' Tools".to_string(), 25),
            ("Healer's Kit".to_string(), 5),
            ("Spell Components".to_string(), 10),
        ];
        
        // Add race-specific gear
        if let Some(race) = &creation_state.selected_race {
            match race.name.as_str() {
                "Dwarf" => {
                    gear.push(("Smith's Tools".to_string(), 20));
                    gear.push(("Mining Pick".to_string(), 2));
                }
                "Elf" => {
                    gear.push(("Elven Cloak".to_string(), 60));
                    gear.push(("Longbow".to_string(), 50));
                }
                "Berserker" => {
                    gear.push(("Two-Handed Sword".to_string(), 30));
                    gear.push(("War Paint".to_string(), 1));
                }
                _ => {}
            }
        }
        
        gear.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by name
        gear
    }
    
    fn apply_character_selections(&self, character: &mut crate::forge::ForgeCharacter, creation_state: &CharacterCreationState) {
        use crate::forge::magic::MagicSchool;
        
        // Apply selected skills and convert magic schools to proper forge magic skills
        for skill in &creation_state.selected_skills {
            match skill.as_str() {
                "Beast Magic" => {
                    character.skills.insert("Beast Magic".to_string(), 1);
                    character.magic.school_skills.insert(MagicSchool::Beast, 1);
                }
                "Elemental Magic" => {
                    character.skills.insert("Elemental Magic".to_string(), 1);
                    character.magic.school_skills.insert(MagicSchool::Elemental, 1);
                }
                "Enchantment Magic" => {
                    character.skills.insert("Enchantment Magic".to_string(), 1);
                    character.magic.school_skills.insert(MagicSchool::Enchantment, 1);
                }
                "Necromancer Magic" => {
                    character.skills.insert("Necromancer Magic".to_string(), 1);
                    character.magic.school_skills.insert(MagicSchool::Necromancer, 1);
                }
                "Divine Magic" => {
                    character.skills.insert("Divine Magic".to_string(), 1);
                    character.magic.school_skills.insert(MagicSchool::Divine, 1);
                }
                _ => {
                    character.skills.insert(skill.clone(), 1); // Other skills start at level 1
                }
            }
        }
        
        // Apply selected spells
        for (spell_name, school) in &creation_state.selected_spells {
            character.magic.add_known_spell(spell_name.clone(), school.clone());
        }
        
        // Apply selected gear to inventory
        for gear in &creation_state.selected_gear {
            // For now, just add gear as misc items until we have proper gear definitions
            let gear_item = crate::forge::InventoryItem {
                name: gear.clone(),
                item_type: crate::forge::ItemType::Misc(crate::forge::MiscItem {
                    misc_type: crate::forge::MiscType::Trade,
                    special_properties: vec!["Starting gear".to_string()],
                }),
                weight: 1.0,
                stack_size: 1,
                quantity: 1,
                value: 10,
                description: "Starting equipment".to_string(),
            };
            character.inventory.add_item(gear_item).ok();
        }
        
        // Set remaining gold (starting gold - spent gold)
        character.gold = creation_state.starting_gold - creation_state.spent_gold;
    }

    fn handle_character_list_input(&mut self, key: KeyEvent, character_list: Vec<(String, chrono::DateTime<chrono::Utc>)>, selected_index: Option<usize>) -> anyhow::Result<()> {
        if character_list.is_empty() {
            // No characters, any key returns to main menu
            self.state = UIState::MainMenu;
            return Ok(());
        }

        match key.code {
            KeyCode::Char('c') | KeyCode::Char('n') => {
                // Create new character
                self.state = UIState::CharacterCreation(crate::ui::CharacterCreationState {
                    step: crate::ui::CreationStep::Rolling,
                    rolled_data: None,
                    selected_race: None,
                    character_name: None,
                    selected_skills: Vec::new(),
                    available_skill_points: 0,
                    selected_spells: Vec::new(),
                    available_spell_picks: 0,
                    selected_gear: Vec::new(),
                    current_selection_index: 0,
                    available_skills_list: Vec::new(),
                    available_spells_list: Vec::new(),
                    available_gear_list: Vec::new(),
                    starting_gold: 100,
                    spent_gold: 0,
                });
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                let new_index = match selected_index {
                    Some(idx) => {
                        if idx > 0 { idx - 1 } else { character_list.len() - 1 }
                    }
                    None => 0,
                };
                self.state = UIState::CharacterList(character_list, Some(new_index));
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                let new_index = match selected_index {
                    Some(idx) => {
                        if idx < character_list.len() - 1 { idx + 1 } else { 0 }
                    }
                    None => 0,
                };
                self.state = UIState::CharacterList(character_list, Some(new_index));
            }
            KeyCode::Enter => {
                if let Some(idx) = selected_index {
                    if idx < character_list.len() {
                        // Sort characters by last played (same as UI)
                        let mut sorted_chars = character_list.clone();
                        sorted_chars.sort_by(|a, b| b.1.cmp(&a.1));
                        
                        let character_name = &sorted_chars[idx].0;

                        match self.state_manager.load_character(character_name) {
                            Ok(mut character) => {
                                character.update_last_played();
                                self.state_manager.update_character(&character)?;
                                self.current_character = Some(character);
                                // Enter world exploration directly at their last position
                                self.enter_world_exploration()?;
                            }
                            Err(_) => {
                                // Failed to load character, return to character list
                                let character_list = self.state_manager.list_characters()?;
                                let selected_index = if character_list.is_empty() { None } else { Some(0) };
                                self.state = UIState::CharacterList(character_list, selected_index);
                            }
                        }
                    }
                }
            }
            KeyCode::Esc => {
                self.state = UIState::MainMenu;
            }
            KeyCode::Char('q') => {
                // We can't return false here as this method returns Result<()>
                // Instead, we'll set a state that the main loop will handle
                self.state = UIState::MainMenu;
            }
            _ => {
                // Any other key, stay in current state
            }
        }
        
        Ok(())
    }

    fn start_combat_encounter(&mut self, character: &ForgeCharacter) -> anyhow::Result<()> {
        // Use tactical combat system
        self.start_tactical_combat_encounter(character)
    }

    fn start_tactical_combat_encounter(&mut self, character: &ForgeCharacter) -> anyhow::Result<()> {
        // Create player tactical combatant
        let mut player_participant = CombatParticipant::from_character(character, Some(Weapon::rusty_sword()));
        player_participant.armor = Some(Armor::leather());
        
        // Calculate movement speed from character's speed characteristic
        let player_movement_speed = character.characteristics.speed as u32;
        
        let player_tactical = crate::forge::TacticalCombatParticipant {
            base_participant: player_participant,
            position: crate::forge::BattlefieldPosition::new(8, 12), // Start player at left side of larger battlefield
            movement_capabilities: crate::forge::MovementCapabilities {
                movement_speed: player_movement_speed,
                can_fly: false,
                can_swim: false,
                can_climb: false,
            },
            movement_remaining: player_movement_speed,
            has_acted: false,
            declared_action: None,
        };
        
        // Generate enemies based on current terrain
        let enemy_participants = self.generate_enemies_for_location()?;
        
        // Convert enemies to tactical participants
        let mut tactical_participants = vec![player_tactical];
        for (i, enemy) in enemy_participants.into_iter().enumerate() {
            // Calculate enemy movement speed from their characteristics (usually 3-4 for most creatures)
            let enemy_movement_speed = 3; // Default for most humanoid enemies
            
            let enemy_tactical = crate::forge::TacticalCombatParticipant {
                base_participant: enemy,
                position: crate::forge::BattlefieldPosition::new(25 + (i as i32) * 3, 10 + (i as i32) * 2), // Start enemies on right side of larger battlefield
                movement_capabilities: crate::forge::MovementCapabilities {
                    movement_speed: enemy_movement_speed,
                    can_fly: false,
                    can_swim: false,
                    can_climb: false,
                },
                movement_remaining: enemy_movement_speed,
                has_acted: false,
                declared_action: None,
            };
            tactical_participants.push(enemy_tactical);
        }
        
        // Create battlefield based on current location - much larger for better tactical gameplay
        let complexity = self.get_battlefield_complexity_for_location();
        let mut battlefield = crate::forge::TacticalBattlefield::generate_battlefield(40, 25, complexity);
        
        // Place participants on battlefield
        for (i, participant) in tactical_participants.iter().enumerate() {
            battlefield.participant_positions.insert(i, participant.position);
        }
        
        // Store reference to current dungeon state if in dungeon
        let return_to_dungeon = if let UIState::DungeonExploration(ref dungeon_state) = self.state {
            Some(dungeon_state.clone())
        } else {
            None
        };
        
        // Create tactical combat state
        let tactical_combat_state = crate::ui::TacticalCombatState::new(
            battlefield,
            tactical_participants,
            return_to_dungeon,
        );

        // DON'T auto-process AI turns - let player see initiative and control flow
        // self.process_ai_turns_until_player(&mut tactical_combat_state)?;

        self.state = UIState::TacticalCombat(tactical_combat_state);
        
        Ok(())
    }

    fn get_battlefield_complexity_for_location(&self) -> crate::forge::BattlefieldComplexity {
        // Get current terrain type if in world exploration
        let terrain_type = if let UIState::WorldExploration(ref world_state) = self.state {
            if let Some(ref zone_data) = world_state.zone_data {
                let local_pos = world_state.player_local_pos;
                zone_data.terrain.tiles[local_pos.y as usize][local_pos.x as usize].terrain_type.clone()
            } else {
                crate::world::terrain::TerrainType::Plains
            }
        } else if let UIState::DungeonExploration(_) = self.state {
            // Dungeons have complex battlefields
            return crate::forge::BattlefieldComplexity::Complex;
        } else {
            crate::world::terrain::TerrainType::Plains
        };
        
        // Return complexity based on terrain
        use crate::world::terrain::TerrainType;
        match terrain_type {
            TerrainType::Plains | TerrainType::Desert | TerrainType::Grassland => crate::forge::BattlefieldComplexity::Simple,
            TerrainType::Forest | TerrainType::Hill => crate::forge::BattlefieldComplexity::Moderate,
            TerrainType::Mountain | TerrainType::Swamp | TerrainType::Snow | TerrainType::Tundra => crate::forge::BattlefieldComplexity::Complex,
            TerrainType::Ocean | TerrainType::Lake | TerrainType::River => crate::forge::BattlefieldComplexity::Moderate, // Water has moderate complexity
        }
    }

    fn generate_enemies_for_location(&self) -> anyhow::Result<Vec<CombatParticipant>> {
        let mut rng = rand::thread_rng();
        let mut enemies = Vec::new();
        
        // Get current terrain type if in world exploration
        let terrain_type = if let UIState::WorldExploration(ref world_state) = self.state {
            if let Some(ref zone_data) = world_state.zone_data {
                let local_pos = world_state.player_local_pos;
                zone_data.terrain.tiles[local_pos.y as usize][local_pos.x as usize].terrain_type.clone()
            } else {
                // Default to plains if no zone data
                crate::world::terrain::TerrainType::Plains
            }
        } else {
            // Default terrain for non-exploration combat
            crate::world::terrain::TerrainType::Plains
        };
        
        // Generate enemies based on terrain
        use crate::world::terrain::TerrainType;
        match terrain_type {
            TerrainType::Forest => {
                // Forest creatures: wolves, spiders, boars
                match rng.gen_range(0..10) {
                    0..=3 => enemies.push(create_wolf()),
                    4..=6 => enemies.push(create_wild_boar()),
                    7..=8 => enemies.push(create_giant_spider()),
                    _ => {
                        // Wolf pack
                        enemies.push(create_wolf());
                        enemies.push(create_wolf());
                    }
                }
            }
            TerrainType::Mountain | TerrainType::Hill => {
                // Mountain creatures: mountain lions, orcs, goblins
                match rng.gen_range(0..10) {
                    0..=2 => enemies.push(create_mountain_lion()),
                    3..=5 => enemies.push(create_goblin()),
                    6..=7 => enemies.push(create_orc()),
                    _ => {
                        // Goblin group
                        enemies.push(create_goblin());
                        enemies.push(create_goblin());
                    }
                }
            }
            TerrainType::Plains | TerrainType::Grassland => {
                // Plains creatures: bandits, wolves, boars
                match rng.gen_range(0..10) {
                    0..=3 => enemies.push(create_bandit()),
                    4..=6 => enemies.push(create_wolf()),
                    7..=8 => enemies.push(create_wild_boar()),
                    _ => {
                        // Bandit group
                        enemies.push(create_bandit());
                        if rng.gen_bool(0.5) {
                            enemies.push(create_bandit());
                        }
                    }
                }
            }
            TerrainType::Swamp => {
                // Swamp creatures: spiders, skeletons
                match rng.gen_range(0..10) {
                    0..=4 => enemies.push(create_giant_spider()),
                    5..=7 => enemies.push(create_skeleton()),
                    _ => {
                        // Spider nest
                        enemies.push(create_giant_spider());
                        enemies.push(create_giant_spider());
                    }
                }
            }
            TerrainType::Desert | TerrainType::Tundra => {
                // Harsh terrain: bandits, skeletons
                match rng.gen_range(0..6) {
                    0..=2 => enemies.push(create_bandit()),
                    _ => enemies.push(create_skeleton()),
                }
            }
            _ => {
                // Default: single wild boar for water/snow/etc
                enemies.push(create_wild_boar());
            }
        }
        
        Ok(enemies)
    }

    #[allow(dead_code)]
    fn skill_requires_target(&self, skill_name: &str) -> bool {
        // Check if this skill requires selecting a target
        match skill_name {
            "Defend" | "Flee" => false,
            _ if skill_name.starts_with("Use ") => false, // Use items typically don't require target selection
            _ => true, // Most combat actions (attacks, spells) require targets
        }
    }
    
    #[allow(dead_code)]
    fn get_available_combat_skills(&self, character: &ForgeCharacter) -> Vec<String> {
        let mut skills = vec!["Basic Attack".to_string()];
        
        // Add character's combat skills
        for (skill_name, &skill_level) in &character.skills {
            if skill_level > 0 {
                match skill_name.as_str() {
                    "Melee Combat" | "Ranged Combat" | "Unarmed Combat" => {
                        skills.push(skill_name.clone());
                    }
                    _ => {}
                }
            }
        }
        
        // Add defensive options
        skills.push("Defend".to_string());
        skills.push("Flee".to_string());
        
        // Add item usage if character has healing items
        if character.inventory.items.iter().any(|item| {
            item.name.contains("Potion") && matches!(item.item_type, crate::forge::ItemType::Consumable(_))
        }) {
            skills.push("Use Item".to_string());
        }
        
        // Add known spells
        let known_spells = character.magic.get_all_known_spells();
        for (_school, spell_name) in known_spells {
            skills.push(format!("Cast {}", spell_name));
        }
        
        skills
    }

    #[allow(dead_code)]
    fn handle_combat_input(&mut self, key: KeyEvent, mut combat_state: CombatState) -> anyhow::Result<()> {
        // Check if combat is over
        if combat_state.encounter.is_combat_over() {
            match key.code {
                KeyCode::Enter => {
                    // Return to dungeon exploration if we came from there
                    // Apply any combat results (XP gain, loot, etc.)
                    if let Some(winner) = combat_state.encounter.get_winner() {
                        if winner == "Player" {
                            self.award_combat_experience(&combat_state)?;
                        }
                    }
                    
                    // Extract defeated enemy information before modifying state
                    let defeated_enemy_names: Vec<String> = combat_state.encounter.participants.iter()
                        .filter(|p| !p.is_player && !p.is_alive())
                        .map(|p| p.name.clone())
                        .collect();
                    
                    if let Some(mut dungeon_state) = combat_state.return_to_dungeon {
                        // Remove defeated enemies from the dungeon floor
                        self.remove_defeated_enemies_by_names(&mut dungeon_state, defeated_enemy_names)?;
                        self.state = UIState::DungeonExploration(dungeon_state);
                    } else {
                        self.state = UIState::Playing;
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        
        // Handle different combat phases
        match combat_state.combat_phase {
            CombatPhase::InitiativeRoll => {
                match key.code {
                    KeyCode::Enter => {
                        combat_state.encounter.add_log("=== COMBAT BEGINS ===".to_string());
                        combat_state.encounter.add_log("Rolling initiative...".to_string());
                        
                        // Display initiative results
                        let init_results: Vec<String> = combat_state.encounter.participants.iter()
                            .map(|p| format!("{} rolled {} for initiative", p.name, p.initiative))
                            .collect();
                        for result in init_results {
                            combat_state.encounter.add_log(result);
                        }
                        
                        combat_state.encounter.add_log(format!("=== ROUND {} ===", combat_state.encounter.round));
                        combat_state.combat_phase = CombatPhase::DeclaringActions;
                    }
                    _ => {}
                }
            }
            CombatPhase::DeclaringActions => {
                match key.code {
                    KeyCode::Enter => {
                        // Start with the first participant (highest initiative)
                        if let Some(current) = combat_state.encounter.get_current_participant() {
                            if current.is_player {
                                combat_state.encounter.add_log(format!("{}'s turn to declare action!", current.name));
                                combat_state.combat_phase = CombatPhase::SelectingSkill;
                            } else {
                                // AI declares action automatically
                                combat_state.encounter.add_log(format!("{} prepares to attack!", current.name));
                                combat_state.encounter.next_turn();
                                
                                // Check if all have declared actions
                                if combat_state.encounter.current_turn == 0 {
                                    combat_state.combat_phase = CombatPhase::ResolvingActions;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        // Handle player's turn for action selection
        if let Some(current) = combat_state.encounter.get_current_participant() {
            if current.is_player && current.is_alive() {
                match combat_state.combat_phase {
                    CombatPhase::SelectingSkill => {
                        match key.code {
                            KeyCode::Up => {
                                if combat_state.current_skill_index > 0 {
                                    combat_state.current_skill_index -= 1;
                                    // Update offset for scrolling
                                    if combat_state.current_skill_index < combat_state.skill_list_offset {
                                        combat_state.skill_list_offset = combat_state.current_skill_index;
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if combat_state.current_skill_index < combat_state.available_skills.len().saturating_sub(1) {
                                    combat_state.current_skill_index += 1;
                                    // Update offset for scrolling
                                    let max_visible = 5;
                                    if combat_state.current_skill_index >= combat_state.skill_list_offset + max_visible {
                                        combat_state.skill_list_offset = combat_state.current_skill_index - max_visible + 1;
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if combat_state.current_skill_index < combat_state.available_skills.len() {
                                    combat_state.selected_skill = Some(combat_state.available_skills[combat_state.current_skill_index].clone());
                                    
                                    // Check if this is a targeted skill/spell
                                    if self.skill_requires_target(&combat_state.available_skills[combat_state.current_skill_index]) {
                                        combat_state.combat_phase = CombatPhase::SelectingTarget;
                                    } else {
                                        // Non-targeted actions (like Defend) proceed directly
                                        let skill_name = combat_state.selected_skill.clone().unwrap_or("Basic Attack".to_string());
                                        
                                        // Execute non-targeted action
                                        let action = match skill_name.as_str() {
                                            "Defend" => CombatAction::Defend,
                                            "Flee" => CombatAction::Flee,
                                            _ => CombatAction::Defend, // Default to defend
                                        };
                                        
                                        let result = combat_state.encounter.perform_action(action);
                                        
                                        // Check if player successfully fled
                                        if skill_name == "Flee" && result.success {
                                            if let Some(dungeon_state) = combat_state.return_to_dungeon {
                                                self.state = UIState::DungeonExploration(dungeon_state);
                                                return Ok(());
                                            } else {
                                                self.state = UIState::Playing;
                                                return Ok(());
                                            }
                                        }
                                        
                                        // Process enemy turns
                                        self.process_ai_turns(&mut combat_state)?;
                                        
                                        // Move to next turn
                                        combat_state.encounter.next_turn();
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                combat_state.selected_skill = None;
                                combat_state.current_skill_index = 0;
                                combat_state.skill_list_offset = 0;
                            }
                            _ => {}
                        }
                    }
                    CombatPhase::SelectingTarget => {
                        match key.code {
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                let target_index = c.to_digit(10).unwrap() as usize - 1;
                                let enemy_count = combat_state.encounter.participants
                                    .iter()
                                    .filter(|p| !p.is_player && p.is_alive())
                                    .count();
                                    
                                if target_index < enemy_count {
                                    // Find the actual target index in the participants list
                                    let mut enemy_counter = 0;
                                    let mut actual_target_index = 0;
                                    
                                    for (i, participant) in combat_state.encounter.participants.iter().enumerate() {
                                        if !participant.is_player && participant.is_alive() {
                                            if enemy_counter == target_index {
                                                actual_target_index = i;
                                                break;
                                            }
                                            enemy_counter += 1;
                                        }
                                    }
                                    
                                    // Execute the skill-based attack or spell
                                    let skill_name = combat_state.selected_skill.clone().unwrap_or("Melee Combat".to_string());
                                    if skill_name.starts_with("Cast ") {
                                        let spell_name = skill_name.strip_prefix("Cast ").unwrap_or(&skill_name);
                                        self.execute_spell_cast(&mut combat_state, actual_target_index, spell_name)?;
                                    } else {
                                        self.execute_skill_attack(&mut combat_state, actual_target_index, &skill_name)?;
                                    }
                                    
                                    combat_state.encounter.next_turn();
                                    combat_state.selected_skill = None;
                                    
                                    // Check if all participants have had their turn
                                    if combat_state.encounter.current_turn == 0 {
                                        // Round complete - start new round
                                        combat_state.encounter.round += 1;
                                        combat_state.encounter.add_log(format!("=== ROUND {} ===", combat_state.encounter.round));
                                        combat_state.combat_phase = CombatPhase::DeclaringActions;
                                    } else {
                                        // Process next participant's turn
                                        self.process_ai_turns(&mut combat_state)?;
                                        
                                        // Check if it's a player's turn again
                                        if let Some(next_participant) = combat_state.encounter.get_current_participant() {
                                            if next_participant.is_player {
                                                combat_state.combat_phase = CombatPhase::SelectingSkill;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                combat_state.combat_phase = CombatPhase::SelectingSkill;
                                combat_state.selected_skill = None;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                
                self.state = UIState::Combat(combat_state);
            }
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    fn execute_skill_attack(&mut self, combat_state: &mut CombatState, target_index: usize, skill_name: &str) -> anyhow::Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let attacker_index = combat_state.encounter.current_turn;
        
        // Get skill and equipment bonuses for the player
        let (skill_bonus, equipment_bonus) = if let Some(character) = &self.current_character {
            use crate::forge::SkillType;
            let skill_bonus = match crate::forge::ForgeCharacter::get_skill_type(skill_name) {
                SkillType::Combat => {
                    let (level, percentage) = character.get_combat_skill_info(skill_name);
                    // Combat skills: Level provides major bonus, percentage provides minor bonus
                    (level as i32 * 5) + (percentage as i32 / 10)
                }
                SkillType::Percentage => {
                    let percentage = character.skills.get(skill_name).copied().unwrap_or(0);
                    // Percentage skills: Every 10% gives +1 bonus
                    percentage as i32 / 10
                }
                SkillType::Magic => 0, // Magic skills don't affect melee combat
            };
            
            // Get equipment bonuses
            let equipment_bonuses = character.get_total_equipment_bonuses();
            
            (skill_bonus, equipment_bonuses.attack_bonus as i32)
        } else {
            (0, 0)
        };
        
        // Get base stats
        let attack_value = combat_state.encounter.participants[attacker_index].get_total_attack_value();
        let defense_value = combat_state.encounter.participants[target_index].get_total_defense_value();
        
        // Roll attack with skill and equipment bonuses
        let attack_roll = rng.gen_range(1..=20);
        let total_bonus = skill_bonus + equipment_bonus;
        let total_attack = attack_roll + attack_value + total_bonus as u8;
        
        let attacker_name = combat_state.encounter.participants[attacker_index].name.clone();
        let target_name = combat_state.encounter.participants[target_index].name.clone();
        
        // Check for critical hit (natural 20)
        let critical = attack_roll == 20;
        
        let skill_display = if let Some(character) = &self.current_character {
            character.get_skill_display(skill_name)
        } else {
            "0%".to_string()
        };
        
        let log_message = format!("{} uses {} ({}) against {}!", 
            attacker_name, skill_name, skill_display, target_name);
        combat_state.encounter.add_log(log_message);
        
        // Check for hit
        if total_attack > defense_value || critical {
            // Roll damage
            let weapon = combat_state.encounter.participants[attacker_index].weapon.clone()
                .unwrap_or_else(Weapon::unarmed);
            let (mut damage, dice_count) = weapon.roll_damage();
            
            // Add damage bonus from character, skill, and equipment
            let character_damage_bonus = combat_state.encounter.participants[attacker_index].get_total_damage_bonus();
            let skill_damage_bonus = if total_bonus >= 5 { 1 } else { 0 }; // Bonus damage at higher skill levels
            let equipment_damage_bonus = if let Some(character) = &self.current_character {
                character.get_total_equipment_bonuses().damage_bonus
            } else { 0 };
            
            let total_damage_bonus = character_damage_bonus + equipment_damage_bonus + skill_damage_bonus as i8;
            
            if total_damage_bonus >= 0 {
                damage += total_damage_bonus as u32;
            } else {
                damage = damage.saturating_sub(total_damage_bonus.abs() as u32);
            }
            
            // Double damage on critical
            let final_dice_count = if critical { dice_count * 2 } else { dice_count };
            if critical {
                damage *= 2;
            }
            
            // Apply damage using Forge rules
            let (actual_damage, armor_damage) = combat_state.encounter.participants[target_index]
                .take_damage(damage, final_dice_count);
            
            let message = if critical {
                format!("CRITICAL HIT! {} damage ({} actual, {} absorbed)!", 
                    damage, actual_damage, armor_damage)
            } else {
                format!("Hit for {} damage ({} actual, {} absorbed)!", 
                    damage, actual_damage, armor_damage)
            };
            
            combat_state.encounter.add_log(message);
            
            // Check if target is defeated
            if !combat_state.encounter.participants[target_index].is_alive() {
                combat_state.encounter.add_log(format!("{} has been defeated!", target_name));
            }
            
            // Award skill pip for successful attack (Traditional Forge advancement)
            let skill_name_clone = skill_name.to_string();
            if let Some(character) = &mut self.current_character {
                let current_pips = character.skill_pips.get(&skill_name_clone).copied().unwrap_or(0);
                let new_pips = current_pips + 1;
                character.skill_pips.insert(skill_name_clone.clone(), new_pips);
                
                // Check if we have enough pips to trigger advancement (minimum 1 pip)
                if new_pips >= 1 {
                    let result = character.advance_skill_with_pips(&skill_name_clone, new_pips);
                    
                    if result.new_value > result.old_value {
                        if result.leveled_up {
                            combat_state.encounter.add_log(format!("🎉 {} skill leveled up! {}% → {}% (Level {})", 
                                result.skill_name, result.old_value, result.new_value, 
                                character.get_combat_skill_info(&skill_name_clone).0));
                        } else {
                            combat_state.encounter.add_log(format!("📈 {} skill improved! {}% → {}%", 
                                result.skill_name, result.old_value, result.new_value));
                        }
                        
                        // Show some of the advancement rolls
                        let success_count = result.rolls.iter().filter(|(_, success)| *success).count();
                        if success_count > 0 {
                            combat_state.encounter.add_log(format!("Advancement rolls: {}/{} successful", 
                                success_count, result.rolls.len()));
                        }
                    }
                }
            }
        } else {
            let message = format!("Attack missed! (rolled {} + {} + {} = {} vs DV {})", 
                attack_roll, attack_value, skill_bonus, total_attack, defense_value);
            combat_state.encounter.add_log(message);
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    fn execute_spell_cast(&mut self, combat_state: &mut CombatState, target_index: usize, spell_name: &str) -> anyhow::Result<()> {
        use rand::Rng;
        
        // Get the spell data
        let spells = crate::forge::magic::create_starter_spells();
        let spell = match spells.get(spell_name) {
            Some(spell) => spell.clone(),
            None => {
                combat_state.encounter.add_log(format!("Unknown spell: {}", spell_name));
                return Ok(());
            }
        };
        
        // Check spell availability and cost first
        let (knows_spell, has_spell_points, school_skill, spell_school) = if let Some(character) = &self.current_character {
            let knows = character.magic.knows_spell(spell_name, &spell.school);
            let has_points = character.magic.can_cast_spell(&spell);
            let skill = character.magic.get_school_skill(&spell.school);
            (knows, has_points, skill, spell.school.clone())
        } else {
            (false, false, 0, spell.school.clone())
        };
        
        if !knows_spell {
            combat_state.encounter.add_log(format!("You don't know the spell: {}", spell_name));
            return Ok(());
        }
        
        if !has_spell_points {
            if let Some(character) = &self.current_character {
                combat_state.encounter.add_log(format!("Not enough spell points to cast {}! ({} required, {} available)", 
                    spell_name, spell.cost, character.magic.spell_points.current));
            }
            return Ok(());
        }
        
        // Spend spell points
        if let Some(character) = &mut self.current_character {
            character.magic.spend_spell_points(spell.cost);
        }
        
        // Calculate success chance and roll
        let success_chance = spell.success_chance_base + (school_skill * 2); // +2% per skill level
        
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(1..=100);
        
        if roll <= success_chance {
            // Spell succeeds!
            combat_state.encounter.add_log(format!("🔮 {} successfully casts {}!", 
                combat_state.encounter.participants[combat_state.encounter.current_turn].name, spell_name));
            
            // Apply spell effects
            for effect in &spell.effects {
                self.apply_spell_effect(combat_state, target_index, effect, spell_name)?;
            }
            
            // Award magic skill advancement
            if let Some(character) = &mut self.current_character {
                let current_skill = character.magic.get_school_skill(&spell_school);
                let current_pips = character.magic.school_pips.get(&spell_school).copied().unwrap_or(0);
                let new_pips = current_pips + 1;
                
                if new_pips >= 10 {
                    // Advance skill level
                    let new_skill = (current_skill + 1).min(20);
                    character.magic.school_skills.insert(spell_school.clone(), new_skill);
                    character.magic.school_pips.insert(spell_school.clone(), 0);
                    
                    combat_state.encounter.add_log(format!("📈 {} advances in {} magic! (Level {})", 
                        character.name, spell_school, new_skill));
                } else {
                    character.magic.school_pips.insert(spell_school.clone(), new_pips);
                }
            }
            
        } else if roll <= success_chance + spell.backfire_chance {
            // Backfire!
            combat_state.encounter.add_log(format!("💥 {} casts {} but it backfires!", 
                combat_state.encounter.participants[combat_state.encounter.current_turn].name, spell_name));
            
            // Simple backfire: take damage
            let backfire_damage = spell.level as u32 * 2;
            let caster_index = combat_state.encounter.current_turn;
            let (actual_damage, _) = combat_state.encounter.participants[caster_index]
                .take_damage(backfire_damage, 1);
            
            combat_state.encounter.add_log(format!("Magical energy courses through your body! {} damage!", actual_damage));
            
        } else {
            // Simple failure
            combat_state.encounter.add_log(format!("❌ {} fails to cast {}.", 
                combat_state.encounter.participants[combat_state.encounter.current_turn].name, spell_name));
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn apply_spell_effect(&mut self, combat_state: &mut CombatState, target_index: usize, effect: &crate::forge::magic::SpellEffect, _spell_name: &str) -> anyhow::Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        match effect {
            crate::forge::magic::SpellEffect::Damage { dice, bonus, damage_type: _ } => {
                // Parse dice string and roll damage
                let damage = if let Some((num_dice, die_size)) = dice.split_once('d') {
                    let dice_count: u32 = num_dice.parse().unwrap_or(1);
                    let die_size: u32 = die_size.parse().unwrap_or(4);
                    
                    let mut total = 0;
                    for _ in 0..dice_count {
                        total += rng.gen_range(1..=die_size);
                    }
                    
                    if *bonus >= 0 {
                        total + (*bonus as u32)
                    } else {
                        total.saturating_sub(bonus.abs() as u32)
                    }
                } else {
                    4 // Default damage
                };
                
                let target_name = combat_state.encounter.participants[target_index].name.clone();
                let (actual_damage, armor_damage) = combat_state.encounter.participants[target_index]
                    .take_damage(damage, 1); // Spells typically pierce some armor
                
                combat_state.encounter.add_log(format!("✨ {} takes {} magical damage ({} actual, {} absorbed)!", 
                    target_name, damage, actual_damage, armor_damage));
                
                if !combat_state.encounter.participants[target_index].is_alive() {
                    combat_state.encounter.add_log(format!("{} has been defeated by magic!", target_name));
                }
            }
            
            crate::forge::magic::SpellEffect::Heal { dice, bonus } => {
                // Parse dice string and roll healing
                let healing = if let Some((num_dice, die_size)) = dice.split_once('d') {
                    let dice_count: u32 = num_dice.parse().unwrap_or(1);
                    let die_size: u32 = die_size.parse().unwrap_or(4);
                    
                    let mut total = 0;
                    for _ in 0..dice_count {
                        total += rng.gen_range(1..=die_size);
                    }
                    
                    if *bonus >= 0 {
                        total + (*bonus as u32)
                    } else {
                        total.saturating_sub(bonus.abs() as u32)
                    }
                } else {
                    4 // Default healing
                };
                
                let target_name = combat_state.encounter.participants[target_index].name.clone();
                combat_state.encounter.participants[target_index].heal(healing);
                
                combat_state.encounter.add_log(format!("💚 {} heals {} for {} points!", 
                    target_name, target_name, healing));
            }
            
            crate::forge::magic::SpellEffect::Buff { stat, modifier, duration } => {
                let target_name = combat_state.encounter.participants[target_index].name.clone();
                combat_state.encounter.add_log(format!("⬆️ {} gains +{} {} for {} rounds!", 
                    target_name, modifier, stat, duration));
                // TODO: Implement buff tracking system
            }
            
            crate::forge::magic::SpellEffect::Debuff { stat, modifier, duration } => {
                let target_name = combat_state.encounter.participants[target_index].name.clone();
                combat_state.encounter.add_log(format!("⬇️ {} suffers {} {} for {} rounds!", 
                    target_name, modifier, stat, duration));
                // TODO: Implement debuff tracking system
            }
            
            crate::forge::magic::SpellEffect::Special { effect, duration: _ } => {
                let target_name = combat_state.encounter.participants[target_index].name.clone();
                combat_state.encounter.add_log(format!("🌟 {}: {}", target_name, effect));
                // TODO: Implement special effect handling
            }
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn award_combat_experience(&mut self, combat_state: &CombatState) -> anyhow::Result<()> {
        if let Some(character) = &mut self.current_character {
            // Award experience based on defeated enemies
            let mut total_xp = 0;
            
            for participant in &combat_state.encounter.participants {
                if !participant.is_player && !participant.is_alive() {
                    // XP based on creature difficulty (HP + attack/defense values)
                    let creature_xp = participant.combat_stats.hit_points.max + 
                        (participant.combat_stats.attack_value as u32) + 
                        (participant.combat_stats.defensive_value as u32);
                    total_xp += creature_xp;
                }
            }
            
            character.experience += total_xp;
            
            // Check for level advancement (simplified)
            let xp_for_next_level = (character.level as u32 + 1) * 100;
            if character.experience >= xp_for_next_level {
                character.level += 1;
                character.experience -= xp_for_next_level;
                
                // Increase hit points on level up
                character.combat_stats.hit_points.max += 5;
                character.combat_stats.hit_points.current = character.combat_stats.hit_points.max;
            }
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    fn remove_defeated_enemies_by_names(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, defeated_enemy_names: Vec<String>) -> anyhow::Result<()> {
        if defeated_enemy_names.is_empty() {
            return Ok(());
        }
        
        // Generate corpses and loot from defeated enemies, then remove them
        if let Some(floor) = dungeon_state.dungeon.get_current_floor_mut() {
            let initial_count = floor.creatures.len();
            let mut corpses_created = 0;
            let mut loot_piles_created = 0;
            
            // Generate corpses and loot for defeated creatures before removing them
            floor.creatures.retain(|creature| {
                if defeated_enemy_names.contains(&creature.name) {
                    // Create corpse at creature's position
                    let mut corpse = crate::world::DungeonCorpse::new(
                        creature.position,
                        creature.creature_type.clone(),
                        creature.name.clone(),
                    );
                    
                    // Generate loot from the corpse
                    let loot_items = corpse.generate_loot();
                    
                    // Add corpse to floor
                    floor.corpses.push(corpse);
                    corpses_created += 1;
                    
                    // Create loot pile if there are items
                    if !loot_items.is_empty() {
                        let loot_pile = crate::world::LootPile {
                            position: creature.position,
                            items: loot_items,
                            source: format!("Corpse of {}", creature.name),
                            discovered: false,
                        };
                        floor.loot_piles.push(loot_pile);
                        loot_piles_created += 1;
                    }
                    
                    false // Remove the creature
                } else {
                    true // Keep the creature
                }
            });
            
            let removed_count = initial_count - floor.creatures.len();
            if removed_count > 0 {
                self.add_dungeon_message(dungeon_state, format!("💀 {} enemies defeated.", removed_count));
                self.add_dungeon_message(dungeon_state, format!("🪦 {} corpses left behind.", corpses_created));
                if loot_piles_created > 0 {
                    self.add_dungeon_message(dungeon_state, format!("💰 {} loot piles created.", loot_piles_created));
                }
            }
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    fn process_ai_turns(&mut self, combat_state: &mut CombatState) -> anyhow::Result<()> {
        loop {
            if combat_state.encounter.is_combat_over() {
                break;
            }
            
            if let Some(current) = combat_state.encounter.get_current_participant() {
                if !current.is_player && current.is_alive() {
                    // Enhanced AI: use personality-based decision making
                    let current_index = combat_state.encounter.current_turn;
                    let action = combat_state.encounter.make_ai_decision(current_index);
                    
                    // Log AI decision for player to see
                    let ai_name = current.name.clone();
                    let action_description = match &action {
                        CombatAction::Attack { .. } => "attacks",
                        CombatAction::Defend => "takes a defensive stance",
                        CombatAction::Flee => "attempts to flee",
                        CombatAction::UseItem { item } => &format!("uses {}", item),
                        CombatAction::CastSpell { spell_name, .. } => &format!("casts {}", spell_name),
                    };
                    
                    combat_state.encounter.add_log(format!("{} {}", ai_name, action_description));
                    
                    // Execute the AI's chosen action
                    let result = combat_state.encounter.perform_action(action.clone());
                    
                    // Handle special cases like fleeing
                    if let CombatAction::Flee = action {
                        if result.success {
                            // AI successfully fled, remove them from combat
                            if let Some(ai_participant) = combat_state.encounter.participants.get_mut(current_index) {
                                ai_participant.combat_stats.hit_points.current = 0; // Mark as "defeated" for simplicity
                            }
                        }
                    }
                    
                    combat_state.encounter.next_turn();
                } else {
                    // It's a player's turn, stop processing
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(())
    }

    fn enter_world_exploration(&mut self) -> anyhow::Result<()> {
        // Initialize world manager if not already done
        if self.world_manager.is_none() {
            let world_name = "default_world";
            let master_seed = 12345; // You could derive this from character or make it configurable
            let save_dir = std::path::Path::new("./world_data");
            
            self.world_manager = Some(WorldManager::new(world_name, master_seed, save_dir)?);
        }
        
        // Load player position from character data if available
        if let Some(character) = &self.current_character {
            if let (Some(zone), Some(position)) = (&character.current_zone, &character.current_position) {
                // Convert zone/local coordinates back to world coordinates
                self.player_position = WorldCoord::from_zone_local(*zone, *position);
            }
        }
        
        // Get current zone and generate if needed
        let current_zone = self.player_position.to_zone();
        let local_pos = self.player_position.to_local();
        
        let zone_data = if let Some(world_manager) = &mut self.world_manager {
            world_manager.get_zone(current_zone).ok().cloned()
        } else {
            None
        };
        
        // Load adjacent zones for seamless world view
        let mut adjacent_zones = std::collections::HashMap::new();
        let adjacent_coords = [
            (current_zone.x - 1, current_zone.y - 1), (current_zone.x, current_zone.y - 1), (current_zone.x + 1, current_zone.y - 1),
            (current_zone.x - 1, current_zone.y),                                            (current_zone.x + 1, current_zone.y),
            (current_zone.x - 1, current_zone.y + 1), (current_zone.x, current_zone.y + 1), (current_zone.x + 1, current_zone.y + 1),
        ];
        
        for (x, y) in adjacent_coords.iter() {
            let adj_coord = crate::world::ZoneCoord::new(*x, *y);
            if let Some(world_manager) = &self.world_manager {
                if let Some(adj_zone) = world_manager.get_zone_if_exists(adj_coord) {
                    adjacent_zones.insert(adj_coord, adj_zone.clone());
                }
            }
        }

        self.state = UIState::WorldExploration(WorldExplorationState {
            current_zone,
            player_local_pos: local_pos,
            zone_data,
            adjacent_zones,
            messages: vec!["Welcome to the world! Press L to look around, H for help, or start exploring with WASD.".to_string()],
        });
        
        Ok(())
    }

    fn handle_world_exploration_input(&mut self, key: KeyEvent, mut world_state: WorldExplorationState) -> anyhow::Result<bool> {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                // Move north
                self.move_player(0, -1, &mut world_state)?;
            }
            KeyCode::Char('s') | KeyCode::Down => {
                // Move south
                self.move_player(0, 1, &mut world_state)?;
            }
            KeyCode::Char('a') | KeyCode::Left => {
                // Move west
                self.move_player(-1, 0, &mut world_state)?;
            }
            KeyCode::Char('d') | KeyCode::Right => {
                // Move east
                self.move_player(1, 0, &mut world_state)?;
            }
            KeyCode::Char('m') => {
                // Return to main menu
                self.state = UIState::Playing;
            }
            KeyCode::Char('b') => {
                // Quick access to inventory (bag)
                if self.current_character.is_some() {
                    let inventory_state = crate::ui::InventoryState {
                        selected_index: 0,
                        scroll_offset: 0,
                        view_mode: crate::ui::InventoryViewMode::List,
                        sort_mode: crate::ui::InventorySortMode::Name,
                        filter_type: None,
                        showing_details: false,
                        selected_item_details: None,
                        sorted_indices: Vec::new(),
                    };
                    self.saved_world_state = Some(world_state.clone());
                    self.state = UIState::InventoryManagement(inventory_state);
                }
            }
            KeyCode::Char('f') => {
                // Start combat at current location
                if let Some(character) = &self.current_character {
                    let character = character.clone();
                    self.start_combat_encounter(&character)?;
                }
            }
            KeyCode::Char('q') => {
                return Ok(true); // Exit game
            }
            KeyCode::Char('e') => {
                // Enter dungeons or examine current location
                if !self.try_enter_dungeon(&mut world_state)? {
                    // If no dungeon to enter, examine location instead
                    self.examine_location(&mut world_state)?;
                }
            }
            KeyCode::Char('t') => {
                // Talk to NPCs at current location
                self.talk_to_npcs(&mut world_state)?;
            }
            KeyCode::Char('r') => {
                // Search current location
                self.search_location(&mut world_state)?;
            }
            KeyCode::Char('i') => {
                // Interact with POIs at current location
                self.interact_with_poi(&mut world_state)?;
            }
            KeyCode::Char('l') => {
                // Look at current tile in detail
                self.look_at_tile(&mut world_state)?;
            }
            KeyCode::Char('c') => {
                // Make camp / rest
                self.make_camp(&mut world_state)?;
            }
            KeyCode::Char('h') => {
                // Show help
                self.show_help(&mut world_state)?;
            }
            KeyCode::Char('g') => {
                // Gather resources
                self.gather_resources(&mut world_state)?;
            }
            KeyCode::Char('p') => {
                // Find nearby POIs
                self.find_nearby_pois(&mut world_state)?;
            }
            // Handle any other character input to prevent random text from appearing
            KeyCode::Char(c) => {
                // Add a message for unrecognized commands
                self.add_message(&mut world_state, format!("Unknown command: '{}'. Press H for help.", c));
            }
            _ => {
                // Ignore all other keys (function keys, special keys, etc.)
            }
        }
        
        // Only update the world state if we're still in world exploration mode
        // (if we entered a dungeon, the state will have changed to DungeonExploration)
        match &self.state {
            UIState::WorldExploration(_) => {
                self.state = UIState::WorldExploration(world_state);
            }
            _ => {
                // State has changed (e.g., entered dungeon), don't overwrite it
            }
        }
        
        Ok(false)
    }

    fn handle_dungeon_exploration_input(&mut self, key: KeyEvent, mut dungeon_state: DungeonExplorationState) -> anyhow::Result<bool> {
        // Tactical combat is now always handled as standalone UIState::TacticalCombat
        // No need to check for embedded combat state here

        // Handle normal dungeon exploration input
        {
            match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                // Move north
                self.move_player_in_dungeon(0, -1, &mut dungeon_state)?;
            }
            KeyCode::Char('s') | KeyCode::Down => {
                // Move south
                self.move_player_in_dungeon(0, 1, &mut dungeon_state)?;
            }
            KeyCode::Char('a') | KeyCode::Left => {
                // Move west
                self.move_player_in_dungeon(-1, 0, &mut dungeon_state)?;
            }
            KeyCode::Char('d') | KeyCode::Right => {
                // Move east
                self.move_player_in_dungeon(1, 0, &mut dungeon_state)?;
            }
            KeyCode::Char('x') => {
                // Exit dungeon - return to world exploration
                self.exit_dungeon(&mut dungeon_state)?;
            }
            KeyCode::Char('u') => {
                // Use stairs
                self.use_stairs(&mut dungeon_state)?;
            }
            KeyCode::Char('e') => {
                // Examine current location
                self.examine_dungeon_location(&mut dungeon_state)?;
            }
            KeyCode::Char('i') => {
                // Interact with features at current location
                self.interact_with_feature(&mut dungeon_state)?;
            }
            KeyCode::Char('l') => {
                // Look at current tile in detail
                self.look_at_dungeon_tile(&mut dungeon_state)?;
            }
            KeyCode::Char('h') => {
                // Show help
                self.show_dungeon_help(&mut dungeon_state)?;
            }
            KeyCode::Char('f') => {
                // Start combat encounter (attack nearby creatures or start random encounter)
                self.initiate_dungeon_combat(&mut dungeon_state)?;
            }
            KeyCode::Char('r') => {
                // Ranged attack - target visible enemies at distance
                self.initiate_ranged_combat(&mut dungeon_state)?;
            }
            KeyCode::Char('t') => {
                // Toggle torch
                self.toggle_torch(&mut dungeon_state)?;
            }
            KeyCode::Char('b') => {
                // Quick access to inventory (bag)
                if self.current_character.is_some() {
                    let inventory_state = crate::ui::InventoryState {
                        selected_index: 0,
                        scroll_offset: 0,
                        view_mode: crate::ui::InventoryViewMode::List,
                        sort_mode: crate::ui::InventorySortMode::Name,
                        filter_type: None,
                        showing_details: false,
                        selected_item_details: None,
                        sorted_indices: Vec::new(),
                    };
                    // Note: We'll need to handle returning to dungeon from inventory
                    self.state = UIState::InventoryManagement(inventory_state);
                }
            }
            KeyCode::Char('q') => {
                return Ok(true); // Exit game
            }
            // Handle any other character input to prevent random text from appearing
            KeyCode::Char(c) => {
                // Check if it's a number key and player is on a corpse
                if c.is_ascii_digit() {
                    let digit = c.to_digit(10).unwrap() as usize;
                    if digit >= 1 && digit <= 9 {
                        self.handle_corpse_number_key(&mut dungeon_state, digit)?;
                    }
                } else {
                    // Add a message for unrecognized commands
                    self.add_dungeon_message(&mut dungeon_state, format!("Unknown command: '{}'. Press H for help.", c));
                }
            }
            _ => {
                // Ignore all other keys (function keys, special keys, etc.)
            }
            }
            
            // Update creatures and game state
            self.update_dungeon_creatures(&mut dungeon_state)?;
        }
        
        // Only update the game state if we're still in dungeon exploration mode
        // (combat might have changed the state to UIState::TacticalCombat)
        if matches!(self.state, UIState::DungeonExploration(_)) {
            self.state = UIState::DungeonExploration(dungeon_state);
        }
        
        Ok(false)
    }

    fn move_player(&mut self, dx: i32, dy: i32, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        let new_local_x = world_state.player_local_pos.x + dx;
        let new_local_y = world_state.player_local_pos.y + dy;
        
        // Check if we need to transition to a new zone
        let mut new_zone = world_state.current_zone;
        let mut final_local_x = new_local_x;
        let mut final_local_y = new_local_y;
        
        if new_local_x < 0 {
            new_zone.x -= 1;
            final_local_x = crate::world::ZONE_SIZE - 1;
        } else if new_local_x >= crate::world::ZONE_SIZE {
            new_zone.x += 1;
            final_local_x = 0;
        }
        
        if new_local_y < 0 {
            new_zone.y -= 1;
            final_local_y = crate::world::ZONE_SIZE - 1;
        } else if new_local_y >= crate::world::ZONE_SIZE {
            new_zone.y += 1;
            final_local_y = 0;
        }
        
        // Generate new zone if we're transitioning
        if new_zone != world_state.current_zone {
            if let Some(world_manager) = &mut self.world_manager {
                world_manager.get_zone(new_zone)?; // Generate if needed
                world_state.zone_data = world_manager.get_zone(new_zone).ok().cloned();
                
                // Load adjacent zones for seamless world view
                world_state.adjacent_zones.clear();
                let adjacent_coords = [
                    (new_zone.x - 1, new_zone.y - 1), (new_zone.x, new_zone.y - 1), (new_zone.x + 1, new_zone.y - 1),
                    (new_zone.x - 1, new_zone.y),                                    (new_zone.x + 1, new_zone.y),
                    (new_zone.x - 1, new_zone.y + 1), (new_zone.x, new_zone.y + 1), (new_zone.x + 1, new_zone.y + 1),
                ];
                
                for (x, y) in adjacent_coords.iter() {
                    let adj_coord = crate::world::ZoneCoord::new(*x, *y);
                    if let Some(adj_zone) = world_manager.get_zone_if_exists(adj_coord) {
                        world_state.adjacent_zones.insert(adj_coord, adj_zone.clone());
                    }
                }
            }
            world_state.current_zone = new_zone;
        } else {
            // Update zone data for current zone if we don't have it
            if world_state.zone_data.is_none() {
                if let Some(world_manager) = &mut self.world_manager {
                    world_state.zone_data = world_manager.get_zone(new_zone).ok().cloned();
                    
                    // Also load adjacent zones if we didn't have zone data
                    if world_state.adjacent_zones.is_empty() {
                        let adjacent_coords = [
                            (new_zone.x - 1, new_zone.y - 1), (new_zone.x, new_zone.y - 1), (new_zone.x + 1, new_zone.y - 1),
                            (new_zone.x - 1, new_zone.y),                                    (new_zone.x + 1, new_zone.y),
                            (new_zone.x - 1, new_zone.y + 1), (new_zone.x, new_zone.y + 1), (new_zone.x + 1, new_zone.y + 1),
                        ];
                        
                        for (x, y) in adjacent_coords.iter() {
                            let adj_coord = crate::world::ZoneCoord::new(*x, *y);
                            if let Some(adj_zone) = world_manager.get_zone_if_exists(adj_coord) {
                                world_state.adjacent_zones.insert(adj_coord, adj_zone.clone());
                            }
                        }
                    }
                }
            }
        }
        
        // Update positions
        world_state.player_local_pos = LocalCoord::new(final_local_x, final_local_y);
        self.player_position = WorldCoord::from_zone_local(new_zone, world_state.player_local_pos);
        
        // Save player position to character data
        if let Some(character) = &mut self.current_character {
            character.current_zone = Some(new_zone);
            character.current_position = Some(world_state.player_local_pos);
        }
        
        // Update the UI state
        self.state = UIState::WorldExploration(world_state.clone());
        
        Ok(())
    }

    fn examine_location(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            let mut examination_text = Vec::new();
            
            // Examine terrain
            if let Some(row) = zone_data.terrain.tiles.get(player_pos.y as usize) {
                if let Some(tile) = row.get(player_pos.x as usize) {
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
                    examination_text.push(format!("You are standing on {}.", terrain_name));
                    examination_text.push(format!("Elevation: {:.1}m, Fertility: {:.1}", tile.elevation * 100.0, tile.fertility));
                }
            }
            
            // Check for NPCs nearby
            let nearby_npcs: Vec<&crate::world::NPC> = zone_data.npcs.iter()
                .filter(|npc| {
                    let dx = (npc.position.x - player_pos.x).abs();
                    let dy = (npc.position.y - player_pos.y).abs();
                    dx <= 2 && dy <= 2
                })
                .collect();
            
            if !nearby_npcs.is_empty() {
                examination_text.push("You see the following people nearby:".to_string());
                for npc in nearby_npcs {
                    examination_text.push(format!("- {} ({})", npc.name, match npc.npc_type {
                        crate::world::NPCType::Merchant => "Merchant",
                        crate::world::NPCType::Guard => "Guard",
                        crate::world::NPCType::Traveler => "Traveler",
                        crate::world::NPCType::Hermit => "Hermit",
                        crate::world::NPCType::Bandit => "Bandit",
                        _ => "Person",
                    }));
                }
            }
            
            // Check for POIs nearby
            let nearby_pois: Vec<&crate::world::PointOfInterest> = zone_data.points_of_interest.iter()
                .filter(|poi| {
                    let dx = (poi.position.x - player_pos.x).abs();
                    let dy = (poi.position.y - player_pos.y).abs();
                    dx <= 3 && dy <= 3
                })
                .collect();
            
            if !nearby_pois.is_empty() {
                examination_text.push("You notice interesting locations nearby:".to_string());
                for poi in nearby_pois {
                    let status = if poi.explored { " (explored)" } else { "" };
                    examination_text.push(format!("- {}: {}{}", poi.name, poi.description, status));
                }
            }
            
            // Add examination results to the message system
            for message in examination_text {
                self.add_message(world_state, message);
            }
        }
        
        Ok(())
    }

    fn talk_to_npcs(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            
            // Find NPCs at the exact same position or adjacent
            let nearby_npcs: Vec<&crate::world::NPC> = zone_data.npcs.iter()
                .filter(|npc| {
                    let dx = (npc.position.x - player_pos.x).abs();
                    let dy = (npc.position.y - player_pos.y).abs();
                    dx <= 1 && dy <= 1
                })
                .collect();
            
            if nearby_npcs.is_empty() {
                self.add_message(world_state, "There's no one here to talk to.".to_string());
            } else {
                // Collect all messages first to avoid borrowing conflicts
                let mut messages = Vec::new();
                
                for npc in nearby_npcs {
                    messages.push(format!("--- Talking to {} ---", npc.name));
                    messages.push(format!("Disposition: {:?}", npc.disposition));
                    for dialogue_line in &npc.dialogue {
                        messages.push(format!("{}: \"{}\"", npc.name, dialogue_line));
                    }
                    
                    if !npc.services.is_empty() {
                        messages.push("Services offered:".to_string());
                        for service in &npc.services {
                            messages.push(format!("- {:?}", service));
                        }
                    }
                    
                    if !npc.inventory.is_empty() {
                        messages.push("Items for trade:".to_string());
                        for item in &npc.inventory {
                            messages.push(format!("- {}", item));
                        }
                    }
                }
                
                // Add all collected messages to the world state
                for message in messages {
                    self.add_message(world_state, message);
                }
            }
        }
        
        Ok(())
    }

    fn search_location(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        let mut messages = Vec::new();
        let mut found_treasure = false;
        
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            
            // Search for hidden treasures in POIs
            for poi in &zone_data.points_of_interest {
                let dx = (poi.position.x - player_pos.x).abs();
                let dy = (poi.position.y - player_pos.y).abs();
                
                if dx <= 2 && dy <= 2 {
                    if let Some(treasure) = &poi.treasure {
                        if treasure.hidden && !poi.explored {
                            messages.push(format!("🔍 You search {} and find hidden treasures!", poi.name));
                            messages.push(format!("💰 Gold: {}", treasure.gold));
                            messages.push(format!("⭐ Experience: {}", treasure.experience));
                            if !treasure.items.is_empty() {
                                messages.push("🎒 Items found:".to_string());
                                for item in &treasure.items {
                                    messages.push(format!("  - {}", item));
                                }
                            }
                            found_treasure = true;
                            
                            // Mark POI as explored
                            // poi.explored = true; // This would require mutable access to zone_data
                        } else if poi.explored {
                            messages.push(format!("You've already searched {} thoroughly.", poi.name));
                        } else if let Some(_treasure) = &poi.treasure {
                            messages.push(format!("You find some treasures at {} that weren't hidden.", poi.name));
                            found_treasure = true;
                        }
                    }
                }
            }
            
            if !found_treasure {
                messages.push("🔍 You search the area but find nothing of interest.".to_string());
            }
        }
        
        // Add all collected messages to the world state
        for message in messages {
            self.add_message(world_state, message);
        }
        
        Ok(())
    }

    fn interact_with_poi(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            
            // Find POIs at current position
            let nearby_pois: Vec<&crate::world::PointOfInterest> = zone_data.points_of_interest.iter()
                .filter(|poi| {
                    let dx = (poi.position.x - player_pos.x).abs();
                    let dy = (poi.position.y - player_pos.y).abs();
                    dx <= 1 && dy <= 1
                })
                .collect();
            
            // Collect all messages first to avoid borrowing conflicts
            let mut messages = Vec::new();
            
            if nearby_pois.is_empty() {
                messages.push("There's nothing special to interact with here.".to_string());
            } else {
                for poi in nearby_pois {
                    messages.push(format!("--- Interacting with {} ---", poi.name));
                    messages.push(poi.description.clone());
                    messages.push(format!("Difficulty: {}/10", poi.difficulty));
                    
                    if let Some(encounter) = &poi.encounter {
                        messages.push(format!("🎲 Encounter: {}", encounter.description));
                        match &encounter.encounter_type {
                            crate::world::EncounterType::Combat(enemies) => {
                                messages.push(format!("⚔️ Prepare for battle against: {}", enemies.join(", ")));
                                // TODO: Start combat encounter
                            }
                            crate::world::EncounterType::Puzzle(puzzle) => {
                                messages.push(format!("🧩 Puzzle: {}", puzzle));
                                messages.push("This requires careful thought to solve...".to_string());
                            }
                            crate::world::EncounterType::Trap(trap) => {
                                messages.push(format!("⚠️ Trap: {}", trap));
                                messages.push("You need to be careful not to trigger it!".to_string());
                            }
                            crate::world::EncounterType::Discovery(discovery) => {
                                messages.push(format!("✨ Discovery: {}", discovery));
                            }
                            crate::world::EncounterType::NPC(npc_name) => {
                                messages.push(format!("👤 You encounter: {}", npc_name));
                            }
                        }
                    }
                    
                    if let Some(treasure) = &poi.treasure {
                        if !treasure.hidden || poi.explored {
                            messages.push("💎 Treasures available:".to_string());
                            messages.push(format!("💰 Gold: {}", treasure.gold));
                            messages.push(format!("⭐ Experience: {}", treasure.experience));
                            if !treasure.items.is_empty() {
                                messages.push("🎒 Items:".to_string());
                                for item in &treasure.items {
                                    messages.push(format!("  - {}", item));
                                }
                            }
                        }
                    }
                    
                    // Check if this POI can be entered as a dungeon
                    if self.can_enter_poi(&poi.poi_type) {
                        messages.push("🚪 Press 'E' to enter this location for detailed exploration!".to_string());
                        
                        // Check if we should auto-enter based on key input
                        // This is a bit of a hack - in a real game we'd want a more elegant input system
                        // For now, we'll add the enter dungeon functionality separately
                    }
                }
            }
            
            // Add all collected messages to the world state
            for message in messages {
                self.add_message(world_state, message);
            }
        }
        
        Ok(())
    }

    fn add_message(&mut self, world_state: &mut WorldExplorationState, message: String) {
        world_state.messages.push(message);
        // Keep only the last 20 messages to prevent memory growth
        if world_state.messages.len() > 20 {
            world_state.messages.remove(0);
        }
        // Update the UI state
        self.state = UIState::WorldExploration(world_state.clone());
    }

    fn look_at_tile(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        // Collect all the data first to avoid borrowing conflicts
        let mut messages = vec!["--- Looking Around ---".to_string()];
        
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            
            // Current tile details
            if let Some(row) = zone_data.terrain.tiles.get(player_pos.y as usize) {
                if let Some(tile) = row.get(player_pos.x as usize) {
                    let terrain_name = match tile.terrain_type {
                        crate::world::TerrainType::Ocean => "Deep Ocean Waters",
                        crate::world::TerrainType::Lake => "Calm Lake",
                        crate::world::TerrainType::River => "Flowing River",
                        crate::world::TerrainType::Plains => "Open Plains",
                        crate::world::TerrainType::Grassland => "Rich Grassland",
                        crate::world::TerrainType::Forest => "Dense Forest",
                        crate::world::TerrainType::Hill => "Rolling Hills",
                        crate::world::TerrainType::Mountain => "Towering Mountain",
                        crate::world::TerrainType::Desert => "Arid Desert",
                        crate::world::TerrainType::Swamp => "Murky Swampland",
                        crate::world::TerrainType::Snow => "Snow-covered Ground",
                        crate::world::TerrainType::Tundra => "Frozen Tundra",
                    };
                    
                    messages.push(format!("🌍 Terrain: {}", terrain_name));
                    messages.push(format!("⛰️ Elevation: {:.1}m | 💧 {:.1}% humidity | 🌡️ {:.1}°C", 
                        tile.elevation * 100.0, tile.moisture * 100.0, (tile.temperature - 0.5) * 40.0));
                    
                    // Terrain-specific descriptions
                    match tile.terrain_type {
                        crate::world::TerrainType::Forest => {
                            messages.push("🌲 Trees sway in the breeze. Birds chirp overhead.".to_string());
                        }
                        crate::world::TerrainType::Desert => {
                            messages.push("🏜️ Sand shifts beneath your feet. The sun beats down.".to_string());
                        }
                        crate::world::TerrainType::Mountain => {
                            messages.push("🏔️ Wind howls through rocky peaks. Air is thin here.".to_string());
                        }
                        crate::world::TerrainType::Swamp => {
                            messages.push("🐸 Strange sounds echo. Ground feels unstable.".to_string());
                        }
                        crate::world::TerrainType::Ocean => {
                            messages.push("🌊 Waves crash nearby. Salty air fills your nose.".to_string());
                        }
                        _ => {}
                    }
                }
            }
            
            // Check for anything special at this exact location
            let at_settlement = zone_data.settlements.iter().find(|s| s.position == player_pos);
            let at_poi = zone_data.points_of_interest.iter().find(|p| p.position == player_pos);
            let at_npc = zone_data.npcs.iter().find(|n| n.position == player_pos);
            
            if let Some(settlement) = at_settlement {
                messages.push(format!("🏘️ You're in {}, a {:?} with {} people.", 
                    settlement.name, settlement.settlement_type, settlement.population));
            }
            
            if let Some(poi) = at_poi {
                messages.push(format!("🏛️ You are at {}!", poi.name));
                messages.push(format!("📖 {}", poi.description));
                if poi.explored {
                    messages.push("✅ You have already explored this location.".to_string());
                } else {
                    messages.push("❓ This location remains unexplored...".to_string());
                }
            }
            
            if let Some(npc) = at_npc {
                messages.push(format!("👤 {} is here with you.", npc.name));
                messages.push(format!("😐 They seem {:?}.", npc.disposition));
            }
            
            // Check roads
            let on_road = zone_data.roads.roads.iter().any(|road| {
                road.path.contains(&player_pos)
            });
            
            if on_road {
                messages.push("🛤️ You are standing on a well-traveled road.".to_string());
            }
            
        } else {
            messages.push("The world is still loading...".to_string());
        }
        
        // Now add all messages at once
        for message in messages {
            self.add_message(world_state, message);
        }
        
        Ok(())
    }

    fn make_camp(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        // Determine safety and gather info first
        let mut messages = vec!["🏕️ Making camp...".to_string()];
        let mut can_camp = true;
        let mut is_safe = true;
        
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            
            // Check terrain safety
            if let Some(row) = zone_data.terrain.tiles.get(player_pos.y as usize) {
                if let Some(tile) = row.get(player_pos.x as usize) {
                    match tile.terrain_type {
                        crate::world::TerrainType::Ocean | crate::world::TerrainType::Lake => {
                            messages.push("❌ You can't camp on water!".to_string());
                            can_camp = false;
                        }
                        crate::world::TerrainType::Mountain => {
                            messages.push("⚠️ Camping on a mountain is dangerous but possible...".to_string());
                            is_safe = false;
                        }
                        crate::world::TerrainType::Swamp => {
                            messages.push("⚠️ The swamp is not an ideal camping spot...".to_string());
                            is_safe = false;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        if !can_camp {
            for message in messages {
                self.add_message(world_state, message);
            }
            return Ok(());
        }
        
        // Handle character healing
        if let Some(character) = &mut self.current_character {
            let hp_recovered = if is_safe { 
                character.combat_stats.hit_points.max / 4 
            } else { 
                character.combat_stats.hit_points.max / 8 
            };
            
            let old_hp = character.combat_stats.hit_points.current;
            character.combat_stats.hit_points.current = 
                (character.combat_stats.hit_points.current + hp_recovered)
                .min(character.combat_stats.hit_points.max);
            
            let actual_recovery = character.combat_stats.hit_points.current - old_hp;
            
            if is_safe {
                messages.push("😴 You set up a comfortable camp and rest peacefully.".to_string());
                messages.push(format!("❤️ You recover {} health points.", actual_recovery));
            } else {
                messages.push("😟 You manage to rest despite the dangerous conditions.".to_string());
                messages.push(format!("❤️ You recover {} health points (reduced).", actual_recovery));
            }
            
            // Small chance of random encounter while camping
            if !is_safe && rand::random::<f32>() < 0.2 {
                messages.push("👹 Your rest is interrupted by a hostile encounter!".to_string());
                // TODO: Trigger random encounter
            }
        }
        
        // Add all messages
        for message in messages {
            self.add_message(world_state, message);
        }
        
        Ok(())
    }

    fn show_help(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        let help_messages = vec![
            "=== WARLORDS HELP ===".to_string(),
            "🗺️ MOVEMENT:".to_string(),
            "  WASD or Arrow Keys - Move around the world".to_string(),
            "  M - Return to main menu".to_string(),
            "  Q - Quit game".to_string(),
            "".to_string(),
            "🔍 EXPLORATION:".to_string(),
            "  L - Look at current tile in detail".to_string(),
            "  E - Enter dungeons OR examine surroundings".to_string(),
            "  P - Find nearby Points of Interest".to_string(),
            "  R - Search for hidden items".to_string(),
            "  I - Interact with Points of Interest".to_string(),
            "".to_string(),
            "👥 SOCIAL:".to_string(),
            "  T - Talk to nearby NPCs".to_string(),
            "".to_string(),
            "⚔️ SURVIVAL:".to_string(),
            "  C - Make camp and rest".to_string(),
            "  F - Fight (start combat encounter)".to_string(),
            "  G - Gather resources".to_string(),
            "  H - Show this help".to_string(),
            "".to_string(),
            "📍 SYMBOLS:".to_string(),
            "  @ - You".to_string(),
            "  █●○◦· - Settlements (Capital/City/Town/Village/Outpost)".to_string(),
            "  MGTHR! - NPCs (Merchant/Guard/Traveler/Hermit/Ranger/Bandit)".to_string(),
            "  ⌂◊♜♠♦ - POIs (Ruins/Cave/Tower/Shrine/Dragon Lair)".to_string(),
            "  ♣^▲.,~ - Terrain (Forest/Hill/Mountain/Plains/Grass/Water)".to_string(),
            "  ═ - Roads".to_string(),
        ];
        
        // Add all help messages to the game state
        for message in help_messages {
            self.add_message(world_state, message);
        }
        
        Ok(())
    }

    fn gather_resources(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        let mut messages = vec!["🔨 Gathering resources...".to_string()];
        
        if let Some(zone_data) = &world_state.zone_data {
            let player_pos = world_state.player_local_pos;
            
            if let Some(row) = zone_data.terrain.tiles.get(player_pos.y as usize) {
                if let Some(tile) = row.get(player_pos.x as usize) {
                    let mut gathered_items = Vec::new();
                    
                    match tile.terrain_type {
                        crate::world::TerrainType::Forest => {
                            gathered_items.extend(["Wood", "Berries", "Medicinal Herbs"]);
                            messages.push("🌲 You gather wood from fallen branches and find some edible berries.".to_string());
                        }
                        crate::world::TerrainType::Mountain => {
                            gathered_items.extend(["Stone", "Iron Ore", "Rare Minerals"]);
                            messages.push("⛏️ You chip away at the rock face and find some useful minerals.".to_string());
                        }
                        crate::world::TerrainType::Plains | crate::world::TerrainType::Grassland => {
                            gathered_items.extend(["Wild Grain", "Flowers", "Small Game"]);
                            messages.push("🌾 You gather wild grains and catch some small game.".to_string());
                        }
                        crate::world::TerrainType::Desert => {
                            gathered_items.extend(["Cactus Water", "Desert Herbs", "Sand"]);
                            messages.push("🌵 You carefully extract water from cacti and find some hardy desert plants.".to_string());
                        }
                        crate::world::TerrainType::Swamp => {
                            gathered_items.extend(["Swamp Moss", "Strange Mushrooms", "Murky Water"]);
                            messages.push("🍄 You collect some unusual swamp vegetation (handle with care!).".to_string());
                        }
                        crate::world::TerrainType::Lake | crate::world::TerrainType::River => {
                            gathered_items.extend(["Fresh Water", "Fish", "Reeds"]);
                            messages.push("🐟 You catch some fish and collect fresh water.".to_string());
                        }
                        crate::world::TerrainType::Snow | crate::world::TerrainType::Tundra => {
                            gathered_items.extend(["Ice", "Arctic Moss", "Animal Tracks"]);
                            messages.push("❄️ You gather some ice and hardy arctic vegetation.".to_string());
                        }
                        _ => {
                            messages.push("🤷 There's nothing useful to gather here.".to_string());
                            // Add all messages collected so far
                            for message in messages {
                                self.add_message(world_state, message);
                            }
                            return Ok(());
                        }
                    }
                    
                    if !gathered_items.is_empty() {
                        messages.push("🎒 Resources gathered:".to_string());
                        for item in gathered_items {
                            messages.push(format!("  - {}", item));
                        }
                        // TODO: Add items to player inventory
                    }
                    
                    // Fertility affects gathering success
                    if tile.fertility > 0.7 {
                        messages.push("✨ The rich environment yields extra resources!".to_string());
                    } else if tile.fertility < 0.3 {
                        messages.push("😞 The poor conditions limit what you can find.".to_string());
                    }
                }
            }
        }
        
        // Add all collected messages to the world state
        for message in messages {
            self.add_message(world_state, message);
        }
        
        Ok(())
    }

    fn can_enter_poi(&self, poi_type: &crate::world::PoiType) -> bool {
        matches!(poi_type,
            crate::world::PoiType::AncientRuins |
            crate::world::PoiType::Cave |
            crate::world::PoiType::AbandonedTower |
            crate::world::PoiType::WizardTower |
            crate::world::PoiType::AbandonedMine |
            crate::world::PoiType::Crypt |
            crate::world::PoiType::Temple |
            crate::world::PoiType::DragonLair |
            crate::world::PoiType::BanditCamp |
            crate::world::PoiType::TreasureVault |
            crate::world::PoiType::Laboratory
        )
    }

    fn try_enter_dungeon(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<bool> {
        let player_pos = world_state.player_local_pos;
        
        if let Some(zone_data) = &world_state.zone_data {
            // Find enterable POIs at current position (exact or adjacent)
            let poi_to_enter = zone_data.points_of_interest.iter()
                .find(|poi| {
                    let dx = (poi.position.x - player_pos.x).abs();
                    let dy = (poi.position.y - player_pos.y).abs();
                    dx <= 1 && dy <= 1 && self.can_enter_poi(&poi.poi_type)
                })
                .cloned();
            
            if let Some(poi) = poi_to_enter {
                self.add_message(world_state, format!("Entering {}...", poi.name));
                self.enter_dungeon(&poi, world_state)?;
                return Ok(true);
            } else {
                // Check if there are enterable POIs nearby but not close enough
                let enterable_nearby = zone_data.points_of_interest.iter()
                    .any(|poi| {
                        let dx = (poi.position.x - player_pos.x).abs();
                        let dy = (poi.position.y - player_pos.y).abs();
                        dx <= 3 && dy <= 3 && self.can_enter_poi(&poi.poi_type)
                    });
                
                if enterable_nearby {
                    self.add_message(world_state, "There are enterable locations nearby. Move closer to a POI and try again.".to_string());
                } else {
                    self.add_message(world_state, "No enterable locations found nearby. Use 'P' to find POIs.".to_string());
                }
            }
        } else {
            self.add_message(world_state, "Zone data not loaded. Cannot check for enterable locations.".to_string());
        }
        
        Ok(false)
    }

    fn enter_dungeon(&mut self, poi: &crate::world::PointOfInterest, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        // Save the current world state so we can restore it when exiting
        self.saved_world_state = Some(world_state.clone());

        // Generate dungeon layout
        let seed = world_state.current_zone.x as u64 * 1000 + world_state.current_zone.y as u64 * 100 + poi.position.x as u64 * 10 + poi.position.y as u64;
        let generator = crate::world::DungeonGenerator::new();
        let dungeon = generator.generate_dungeon(poi.poi_type.clone(), poi.name.clone(), seed);

        // Create dungeon exploration state
        let mut dungeon_state = crate::ui::DungeonExplorationState {
            dungeon,
            player_pos: crate::world::LocalCoord::new(crate::world::DUNGEON_WIDTH / 2, crate::world::DUNGEON_HEIGHT - 2), // Entrance
            messages: vec![
                format!("You enter {}...", poi.name),
                "The air grows thick as you step inside.".to_string(),
                "Type 'H' for help with dungeon exploration.".to_string(),
            ],
            turn_count: 0,
        };

        // Calculate initial visibility based on character's vision radius and torch status
        self.update_visibility(&mut dungeon_state);

        // Switch to dungeon exploration mode
        self.state = crate::ui::UIState::DungeonExploration(dungeon_state);

        Ok(())
    }

    fn move_player_in_dungeon(&mut self, dx: i32, dy: i32, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let new_x = dungeon_state.player_pos.x + dx;
        let new_y = dungeon_state.player_pos.y + dy;
        
        // Check bounds
        if new_x < 0 || new_x >= crate::world::DUNGEON_WIDTH || new_y < 0 || new_y >= crate::world::DUNGEON_HEIGHT {
            self.add_dungeon_message(dungeon_state, "You can't go that way.".to_string());
            return Ok(());
        }
        
        // Check if the destination tile is passable
        if let Some(tile) = dungeon_state.dungeon.get_tile_at(crate::world::LocalCoord::new(new_x, new_y)) {
            let can_move = match &tile.tile_type {
                crate::world::DungeonTileType::Floor |
                crate::world::DungeonTileType::Stairs(_) |
                crate::world::DungeonTileType::Chest |
                crate::world::DungeonTileType::Altar |
                crate::world::DungeonTileType::Torch => true,
                crate::world::DungeonTileType::Door(state) => {
                    match state {
                        crate::world::DoorState::Open => true,
                        crate::world::DoorState::Closed => {
                            self.add_dungeon_message(dungeon_state, "The door is closed. Try interacting with it.".to_string());
                            false
                        },
                        crate::world::DoorState::Locked => {
                            self.add_dungeon_message(dungeon_state, "The door is locked.".to_string());
                            false
                        },
                        crate::world::DoorState::Secret => {
                            self.add_dungeon_message(dungeon_state, "You feel like there might be something hidden here...".to_string());
                            false
                        },
                    }
                },
                crate::world::DungeonTileType::Water => {
                    self.add_dungeon_message(dungeon_state, "You wade through the shallow water.".to_string());
                    true
                },
                _ => {
                    self.add_dungeon_message(dungeon_state, "You can't move there.".to_string());
                    false
                }
            };
            
            if can_move {
                // Check for creatures at destination
                if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
                    if let Some(creature) = floor.creatures.iter().find(|c| c.position.x == new_x && c.position.y == new_y) {
                        self.add_dungeon_message(dungeon_state, format!("A {} blocks your path!", creature.name));
                        return Ok(());
                    }
                }
                
                // Move player
                dungeon_state.player_pos = crate::world::LocalCoord::new(new_x, new_y);
                dungeon_state.turn_count += 1;
                
                // Advance corpse decay every 5 turns
                if dungeon_state.turn_count % 5 == 0 {
                    self.advance_corpse_decay(dungeon_state);
                }
                
                // Update visibility around player
                self.update_visibility(dungeon_state);
                
                // Check for enemy aggro (automatic combat initiation)
                if self.check_enemy_aggro(dungeon_state)? {
                    // Combat was initiated, return early
                    return Ok(());
                }
                
                // Check for automatic interactions
                self.check_automatic_interactions(dungeon_state)?;
            }
        }
        
        Ok(())
    }

    fn update_visibility(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) {
        let player_pos = dungeon_state.player_pos;
        let visibility_radius = if let Some(character) = &self.current_character {
            character.get_vision_radius() as i32
        } else {
            3 // Default fallback
        };
        
        if let Some(floor) = dungeon_state.dungeon.get_current_floor_mut() {
            // Reset visibility
            for row in &mut floor.tiles {
                for tile in row {
                    tile.visible = false;
                }
            }
            
            // Set visibility around player
            for dy in -visibility_radius..=visibility_radius {
                for dx in -visibility_radius..=visibility_radius {
                    let x = player_pos.x + dx;
                    let y = player_pos.y + dy;
                    
                    if x >= 0 && x < crate::world::DUNGEON_WIDTH && y >= 0 && y < crate::world::DUNGEON_HEIGHT {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance <= visibility_radius as f32 {
                            if let Some(tile) = floor.tiles.get_mut(y as usize).and_then(|row| row.get_mut(x as usize)) {
                                tile.visible = true;
                                tile.explored = true;
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_enemy_aggro(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<bool> {
        let player_pos = dungeon_state.player_pos;
        let aggro_range = 2; // Enemies attack when player gets within 2 tiles
        
        // Find visible enemies within aggro range - collect info first to avoid borrow issues
        let aggro_creature = if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            let mut found_creature = None;
            
            for creature in &floor.creatures {
                let dx = (creature.position.x - player_pos.x).abs();
                let dy = (creature.position.y - player_pos.y).abs();
                let distance = dx.max(dy); // Chebyshev distance (allows diagonal movement)
                
                if distance <= aggro_range {
                    // Check if the creature's tile is visible
                    if let Some(tile) = floor.tiles.get(creature.position.y as usize)
                        .and_then(|row| row.get(creature.position.x as usize)) {
                        if tile.visible {
                            found_creature = Some(creature.clone());
                            break;
                        }
                    }
                }
            }
            
            found_creature
        } else {
            None
        };
        
        // If we found an aggro creature, start combat
        if let Some(creature) = aggro_creature {
            self.add_dungeon_message(dungeon_state, format!("🚨 {} notices you and attacks!", creature.name));
            self.start_dungeon_combat(dungeon_state, &creature)?;
            return Ok(true); // Combat started
        }
        
        Ok(false) // No combat started
    }

    fn check_automatic_interactions(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        
        if let Some(tile) = dungeon_state.dungeon.get_tile_at(player_pos) {
            match &tile.tile_type {
                crate::world::DungeonTileType::Stairs(stair_type) => {
                    match stair_type {
                        crate::world::StairType::Up => {
                            self.add_dungeon_message(dungeon_state, "You see stairs leading up. Press 'U' to use them.".to_string());
                        },
                        crate::world::StairType::Down => {
                            self.add_dungeon_message(dungeon_state, "You see stairs leading down. Press 'U' to use them.".to_string());
                        },
                        crate::world::StairType::UpDown => {
                            self.add_dungeon_message(dungeon_state, "You see a spiral staircase. Press 'U' to use it.".to_string());
                        },
                    }
                },
                crate::world::DungeonTileType::Chest => {
                    self.add_dungeon_message(dungeon_state, "You see a treasure chest! Press 'I' to interact with it.".to_string());
                },
                crate::world::DungeonTileType::Altar => {
                    self.add_dungeon_message(dungeon_state, "An ancient altar stands before you. Press 'I' to examine it.".to_string());
                },
                _ => {}
            }
        }
        
        // Check for features at current position
        if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            if let Some(feature) = floor.features.iter().find(|f| f.position == player_pos) {
                self.add_dungeon_message(dungeon_state, format!("You notice: {}", feature.description));
            }
        }
        
        Ok(())
    }

    fn update_dungeon_creatures(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let turn = dungeon_state.turn_count;
        
        if let Some(floor) = dungeon_state.dungeon.get_current_floor_mut() {
            for creature in &mut floor.creatures {
                // Update creature movement based on cooldown
                if turn >= creature.last_move_time + creature.movement_cooldown {
                    creature.last_move_time = turn;
                    
                    // Simple AI: move along patrol route
                    if !creature.patrol_route.is_empty() {
                        creature.current_patrol_index = (creature.current_patrol_index + 1) % creature.patrol_route.len();
                        let target = creature.patrol_route[creature.current_patrol_index];
                        
                        // Move towards patrol point
                        if creature.position.x < target.x { creature.position.x += 1; }
                        else if creature.position.x > target.x { creature.position.x -= 1; }
                        else if creature.position.y < target.y { creature.position.y += 1; }
                        else if creature.position.y > target.y { creature.position.y -= 1; }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn exit_dungeon(&mut self, _dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        // Restore the saved world state
        if let Some(mut world_state) = self.saved_world_state.take() {
            // Add an exit message
            world_state.messages.push("You exit the dungeon and return to the world.".to_string());
            
            // Keep only the last 20 messages to prevent memory growth
            if world_state.messages.len() > 20 {
                world_state.messages.remove(0);
            }
            
            self.state = crate::ui::UIState::WorldExploration(world_state);
        } else {
            // Fallback if no saved state (shouldn't happen)
            let world_state = crate::ui::WorldExplorationState {
                current_zone: crate::world::ZoneCoord::new(4, 4), // Default center
                player_local_pos: crate::world::LocalCoord::new(32, 32),
                zone_data: None, // Will be regenerated
                adjacent_zones: std::collections::HashMap::new(), // Empty - will be populated when zone loads
                messages: vec!["You exit the dungeon and return to the world.".to_string()],
            };
            
            self.state = crate::ui::UIState::WorldExploration(world_state);
        }
        
        Ok(())
    }

    fn use_stairs(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        
        if let Some(tile) = dungeon_state.dungeon.get_tile_at(player_pos) {
            if let crate::world::DungeonTileType::Stairs(stair_type) = &tile.tile_type {
                match stair_type {
                    crate::world::StairType::Up => {
                        if dungeon_state.dungeon.current_floor > 0 {
                            dungeon_state.dungeon.current_floor -= 1;
                            self.add_dungeon_message(dungeon_state, format!("You climb up to floor {}.", dungeon_state.dungeon.current_floor + 1));
                        } else {
                            self.add_dungeon_message(dungeon_state, "You can't go up any further.".to_string());
                        }
                    },
                    crate::world::StairType::Down => {
                        let max_floor = dungeon_state.dungeon.floors.len() as i32 - 1;
                        if dungeon_state.dungeon.current_floor < max_floor {
                            dungeon_state.dungeon.current_floor += 1;
                            self.add_dungeon_message(dungeon_state, format!("You descend to floor {}.", dungeon_state.dungeon.current_floor + 1));
                        } else {
                            self.add_dungeon_message(dungeon_state, "The stairs end here.".to_string());
                        }
                    },
                    crate::world::StairType::UpDown => {
                        // For spiral staircases, allow choosing direction
                        self.add_dungeon_message(dungeon_state, "This staircase goes both ways. Use 'U' again to go up, or move to go down.".to_string());
                    },
                }
                
                // Update visibility after floor change
                self.update_visibility(dungeon_state);
            } else {
                self.add_dungeon_message(dungeon_state, "There are no stairs here.".to_string());
            }
        }
        
        Ok(())
    }

    fn examine_dungeon_location(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        let mut messages = Vec::new();
        
        if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            messages.push(format!("=== Floor {} - Position ({}, {}) ===", 
                dungeon_state.dungeon.current_floor + 1, player_pos.x, player_pos.y));
            
            // Describe current tile
            if let Some(tile) = floor.tiles.get(player_pos.y as usize).and_then(|row| row.get(player_pos.x as usize)) {
                let description = match &tile.tile_type {
                    crate::world::DungeonTileType::Floor => "You stand on stone flooring.",
                    crate::world::DungeonTileType::Stairs(stair_type) => {
                        match stair_type {
                            crate::world::StairType::Up => "Stone steps lead upward.",
                            crate::world::StairType::Down => "Stone steps descend into darkness.",
                            crate::world::StairType::UpDown => "A spiral staircase winds both up and down.",
                        }
                    },
                    crate::world::DungeonTileType::Door(_) => "An ancient door stands before you.",
                    crate::world::DungeonTileType::Chest => "A treasure chest sits here, waiting to be opened.",
                    crate::world::DungeonTileType::Altar => "An ornate altar dominates this space.",
                    crate::world::DungeonTileType::Pillar => "A stone pillar supports the ceiling here.",
                    crate::world::DungeonTileType::Water => "Shallow water pools on the floor.",
                    crate::world::DungeonTileType::Rubble => "Chunks of stone and debris litter the ground.",
                    _ => "The details of this area are unclear in the dim light.",
                };
                messages.push(description.to_string());
                
                if tile.light_level > 5 {
                    messages.push("The area is well-lit.".to_string());
                } else if tile.light_level > 2 {
                    messages.push("Dim light illuminates the surroundings.".to_string());
                } else {
                    messages.push("The area is shrouded in darkness.".to_string());
                }
            }
            
            // Look for creatures in view
            let visible_creatures: Vec<&crate::world::DungeonCreature> = floor.creatures.iter()
                .filter(|creature| {
                    let dx = (creature.position.x - player_pos.x).abs();
                    let dy = (creature.position.y - player_pos.y).abs();
                    dx <= 3 && dy <= 3 // Within visibility range
                })
                .collect();
            
            if !visible_creatures.is_empty() {
                messages.push("Creatures in sight:".to_string());
                for creature in visible_creatures {
                    let distance = ((creature.position.x - player_pos.x).pow(2) + (creature.position.y - player_pos.y).pow(2) as i32).abs();
                    messages.push(format!("  {} (distance: {})", creature.name, distance));
                }
            }
            
            // Look for features
            let nearby_features: Vec<&crate::world::DungeonFeature> = floor.features.iter()
                .filter(|feature| {
                    let dx = (feature.position.x - player_pos.x).abs();
                    let dy = (feature.position.y - player_pos.y).abs();
                    dx <= 1 && dy <= 1
                })
                .collect();
            
            if !nearby_features.is_empty() {
                messages.push("Notable features:".to_string());
                for feature in nearby_features {
                    messages.push(format!("  {}", feature.description));
                }
            }
        }
        
        for message in messages {
            self.add_dungeon_message(dungeon_state, message);
        }
        
        Ok(())
    }

    fn interact_with_feature(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        
        // Check current tile for interactions
        if let Some(tile) = dungeon_state.dungeon.get_tile_at(player_pos) {
            match &tile.tile_type {
                crate::world::DungeonTileType::Chest => {
                    self.add_dungeon_message(dungeon_state, "You open the treasure chest!".to_string());
                    self.add_dungeon_message(dungeon_state, "Inside you find: Gold coins, a health potion, and an ancient scroll.".to_string());
                },
                crate::world::DungeonTileType::Door(state) => {
                    match state {
                        crate::world::DoorState::Closed => {
                            self.add_dungeon_message(dungeon_state, "You push open the door.".to_string());
                            // TODO: Actually change door state to open
                        },
                        crate::world::DoorState::Open => {
                            self.add_dungeon_message(dungeon_state, "The door is already open.".to_string());
                        },
                        crate::world::DoorState::Locked => {
                            self.add_dungeon_message(dungeon_state, "The door is locked. You need a key.".to_string());
                        },
                        crate::world::DoorState::Secret => {
                            self.add_dungeon_message(dungeon_state, "You search carefully and find a hidden mechanism!".to_string());
                            // TODO: Reveal secret door
                        },
                    }
                },
                crate::world::DungeonTileType::Altar => {
                    self.add_dungeon_message(dungeon_state, "You examine the ancient altar. Ancient runes glow faintly as you approach.".to_string());
                    self.add_dungeon_message(dungeon_state, "You feel a mysterious energy emanating from it.".to_string());
                },
                _ => {
                    // Check for features at this position
                    if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
                        if let Some(feature) = floor.features.iter().find(|f| f.position == player_pos) {
                            match &feature.feature_type {
                                crate::world::FeatureType::Bookshelf => {
                                    self.add_dungeon_message(dungeon_state, "You browse the ancient books. Most are too damaged to read, but you find a useful spell scroll.".to_string());
                                },
                                crate::world::FeatureType::WeaponRack => {
                                    self.add_dungeon_message(dungeon_state, "You examine the weapon rack. Some rusty weapons remain, but one sword looks usable.".to_string());
                                },
                                crate::world::FeatureType::ArmorStand => {
                                    self.add_dungeon_message(dungeon_state, "You inspect the armor stand. The chainmail appears to be in good condition.".to_string());
                                },
                                crate::world::FeatureType::Lever => {
                                    self.add_dungeon_message(dungeon_state, "You pull the lever. You hear a distant rumbling...".to_string());
                                },
                                crate::world::FeatureType::Crystal => {
                                    self.add_dungeon_message(dungeon_state, "The crystal pulses with magical energy. You feel refreshed!".to_string());
                                },
                                crate::world::FeatureType::Statue => {
                                    self.add_dungeon_message(dungeon_state, "You examine the statue. It depicts a forgotten hero from ages past.".to_string());
                                },
                                _ => {
                                    self.add_dungeon_message(dungeon_state, feature.description.clone());
                                }
                            }
                        } else {
                            // Check for corpses at this position
                            let corpse_found = floor.corpses.iter().find(|c| c.position == player_pos).cloned();
                            let loot_pile_found = floor.loot_piles.iter().find(|lp| lp.position == player_pos).cloned();
                            
                            if let Some(corpse) = corpse_found {
                                self.interact_with_corpse(dungeon_state, &corpse)?;
                            } else if let Some(loot_pile) = loot_pile_found {
                                self.interact_with_loot_pile(dungeon_state, &loot_pile)?;
                            } else {
                                self.add_dungeon_message(dungeon_state, "There's nothing special to interact with here.".to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn look_at_dungeon_tile(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        self.examine_dungeon_location(dungeon_state)
    }

    fn show_dungeon_help(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let help_messages = vec![
            "=== DUNGEON EXPLORATION HELP ===".to_string(),
            "Movement: W/A/S/D or Arrow Keys".to_string(),
            "E - Examine current location in detail".to_string(),
            "I - Interact with objects and features".to_string(),
            "U - Use stairs to change floors".to_string(),
            "F - Attack nearby creatures (melee)".to_string(),
            "R - Ranged attack (spells/arrows at distance)".to_string(),
            "T - Toggle torch (light/extinguish)".to_string(),
            "L - Look around (same as examine)".to_string(),
            "X - Exit dungeon and return to world".to_string(),
            "H - Show this help".to_string(),
            "Ctrl+Q - Quit game".to_string(),
            "".to_string(),
            "Symbols:".to_string(),
            "@  - You        # - Wall      . - Floor".to_string(),
            "+  - Open Door  | - Closed Door".to_string(),
            "<  - Stairs Up  > - Stairs Down".to_string(),
            "C  - Chest      A - Altar     I - Pillar".to_string(),
            "S  - Skeleton   Z - Zombie    G - Ghost".to_string(),
            "b  - Bat        r - Rat       s - Spider".to_string(),
            "g  - Goblin     O - Orc       B - Bandit".to_string(),
            "%  - Corpse     $  - Loot     ?  - Undiscovered Loot".to_string(),
        ];
        
        for message in help_messages {
            self.add_dungeon_message(dungeon_state, message);
        }
        
        Ok(())
    }

    fn toggle_torch(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        if let Some(character) = &mut self.current_character {
            if character.torch_lit {
                // Extinguish torch
                character.extinguish_torch();
                self.add_dungeon_message(dungeon_state, "You extinguish your torch.".to_string());
            } else {
                // Try to light torch
                if character.light_torch() {
                    self.add_dungeon_message(dungeon_state, "You light a torch. Your vision extends!".to_string());
                } else {
                    self.add_dungeon_message(dungeon_state, "You don't have any torches to light.".to_string());
                }
            }
            
            // Update visibility with new vision radius
            self.update_visibility(dungeon_state);
        }
        
        // Update the state to maintain UI consistency
        if matches!(self.state, UIState::DungeonExploration(_)) {
            self.state = UIState::DungeonExploration(dungeon_state.clone());
        }
        
        Ok(())
    }

    fn interact_with_corpse(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, corpse: &crate::world::DungeonCorpse) -> anyhow::Result<()> {
        self.add_dungeon_message(dungeon_state, format!("🪦 You examine the corpse of {}.", corpse.name));
        
        // Show available interactions
        let mut interaction_messages = vec!["Available actions:".to_string()];
        for (i, interaction) in corpse.interactions.iter().enumerate() {
            let description = match interaction {
                crate::world::CorpseInteraction::Loot => "Loot - Search for items and gold",
                crate::world::CorpseInteraction::Skin => "Skin - Harvest hide and meat",
                crate::world::CorpseInteraction::Harvest => "Harvest - Collect magical components",
                crate::world::CorpseInteraction::RaiseSkeleton => "Raise Skeleton - Necromancy spell",
                crate::world::CorpseInteraction::RaiseZombie => "Raise Zombie - Necromancy spell",
                crate::world::CorpseInteraction::Examine => "Examine - Study the corpse closely",
                crate::world::CorpseInteraction::Burn => "Burn - Destroy the corpse",
            };
            interaction_messages.push(format!("  {} - {}", i + 1, description));
        }
        interaction_messages.push("Press I again to select an action...".to_string());
        
        for message in interaction_messages {
            self.add_dungeon_message(dungeon_state, message);
        }
        
        // TODO: Implement action selection UI
        // For now, just auto-loot if possible
        if corpse.interactions.contains(&crate::world::CorpseInteraction::Loot) && !corpse.loot_generated {
            self.auto_loot_corpse(dungeon_state, corpse.position)?;
        }
        
        Ok(())
    }
    
    fn interact_with_loot_pile(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, loot_pile: &crate::world::LootPile) -> anyhow::Result<()> {
        self.add_dungeon_message(dungeon_state, format!("💰 You find a loot pile: {}", loot_pile.source));
        
        if loot_pile.items.is_empty() {
            self.add_dungeon_message(dungeon_state, "The pile is empty.".to_string());
            return Ok(());
        }
        
        self.add_dungeon_message(dungeon_state, "Items found:".to_string());
        for item in &loot_pile.items {
            let item_desc = if item.quantity > 1 {
                format!("  {} x{} ({}gp each) - {}", item.name, item.quantity, item.value, item.description)
            } else {
                format!("  {} ({}gp) - {}", item.name, item.value, item.description)
            };
            self.add_dungeon_message(dungeon_state, item_desc);
        }
        
        // TODO: Implement item selection UI
        // For now, auto-take all items
        self.auto_take_loot(dungeon_state, loot_pile)?;
        
        Ok(())
    }
    
    fn auto_loot_corpse(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, corpse_position: crate::world::LocalCoord) -> anyhow::Result<()> {
        // Find and mutate the corpse at the given position
        if let Some(floor) = dungeon_state.dungeon.get_current_floor_mut() {
            if let Some(corpse) = floor.corpses.iter_mut().find(|c| c.position == corpse_position) {
                // Check if already looted
                if corpse.loot_generated {
                    self.add_dungeon_message(dungeon_state, "This corpse has already been looted.".to_string());
                    return Ok(());
                }
                
                let loot_items = corpse.generate_loot();
                
                if loot_items.is_empty() {
                    self.add_dungeon_message(dungeon_state, "You find nothing of value on the corpse.".to_string());
                } else {
                    self.add_dungeon_message(dungeon_state, "You loot the corpse and find:".to_string());
                    let mut total_gold = 0u32;
                    
                    for item in &loot_items {
                        match item.item_type {
                            crate::world::LootItemType::Gold => {
                                total_gold += item.quantity * item.value;
                            }
                            _ => {
                                let item_desc = if item.quantity > 1 {
                                    format!("  {} x{}", item.name, item.quantity)
                                } else {
                                    format!("  {}", item.name)
                                };
                                self.add_dungeon_message(dungeon_state, item_desc);
                                
                                // Add to character inventory
                                if let Some(character) = &mut self.current_character {
                                    // Convert loot to inventory item
                                    let inventory_item = crate::forge::InventoryItem {
                                        name: item.name.clone(),
                                        item_type: crate::forge::ItemType::Misc(crate::forge::MiscItem {
                                            misc_type: crate::forge::MiscType::Trade,
                                            special_properties: vec!["Dungeon loot".to_string()],
                                        }),
                                        weight: 0.5,
                                        stack_size: 1,
                                        quantity: 1,
                                        value: 5,
                                        description: "Loot found in dungeon".to_string(),
                                    };
                                    character.inventory.add_item(inventory_item).ok();
                                }
                            }
                        }
                    }
                    
                    if total_gold > 0 {
                        self.add_dungeon_message(dungeon_state, format!("  {} gold coins", total_gold));
                        // Add gold to character
                        if let Some(character) = &mut self.current_character {
                            character.gold += total_gold;
                        }
                    }
                }
            } else {
                self.add_dungeon_message(dungeon_state, "No corpse found at this location.".to_string());
            }
        }
        
        Ok(())
    }
    
    fn auto_take_loot(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, loot_pile: &crate::world::LootPile) -> anyhow::Result<()> {
        self.add_dungeon_message(dungeon_state, "You take all the items.".to_string());
        let mut total_gold = 0u32;
        
        for item in &loot_pile.items {
            match item.item_type {
                crate::world::LootItemType::Gold => {
                    total_gold += item.quantity * item.value;
                }
                _ => {
                    // Add to character inventory
                    if let Some(character) = &mut self.current_character {
                        let inventory_item = crate::forge::InventoryItem {
                            name: item.name.clone(),
                            item_type: crate::forge::ItemType::Misc(crate::forge::MiscItem {
                                misc_type: crate::forge::MiscType::Trade,
                                special_properties: vec!["Loot pile".to_string()],
                            }),
                            weight: 0.5,
                            stack_size: 99,
                            quantity: item.quantity,
                            value: item.value,
                            description: "Items found in a loot pile".to_string(),
                        };
                        character.inventory.add_item(inventory_item).ok();
                    }
                }
            }
        }
        
        if total_gold > 0 {
            // Add gold to character
            if let Some(character) = &mut self.current_character {
                character.gold += total_gold;
                self.add_dungeon_message(dungeon_state, format!("💰 You gained {} gold!", total_gold));
            }
        }
        
        // TODO: Remove the loot pile from the floor after taking items
        
        Ok(())
    }

    fn advance_corpse_decay(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) {
        if let Some(floor) = dungeon_state.dungeon.get_current_floor_mut() {
            let mut decay_messages = Vec::new();
            let mut corpses_to_remove = Vec::new();
            
            for (i, corpse) in floor.corpses.iter_mut().enumerate() {
                let old_decay = corpse.decay_level;
                corpse.advance_decay();
                
                // Check for decay level changes that should be reported
                match (old_decay, corpse.decay_level) {
                    (0..=2, 3..=5) => {
                        decay_messages.push(format!("The corpse of {} is starting to decay.", corpse.name));
                    }
                    (3..=5, 6..=9) => {
                        decay_messages.push(format!("The remains of {} have turned skeletal.", corpse.name));
                    }
                    (6..=9, 10) => {
                        decay_messages.push(format!("The bones of {} crumble to dust.", corpse.name));
                        corpses_to_remove.push(i);
                    }
                    _ => {}
                }
            }
            
            // Remove completely decayed corpses (in reverse order to maintain indices)
            for &index in corpses_to_remove.iter().rev() {
                floor.corpses.remove(index);
            }
            
            // Show decay messages to player
            for message in decay_messages {
                self.add_dungeon_message(dungeon_state, message);
            }
        }
    }

    fn handle_corpse_number_key(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, action_number: usize) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        
        // Check if player is standing on a corpse
        if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            if let Some(corpse) = floor.corpses.iter().find(|c| c.position == player_pos) {
                // Check if the action number is valid
                if action_number > 0 && action_number <= corpse.interactions.len() {
                    let action = &corpse.interactions[action_number - 1];
                    let corpse_name = corpse.name.clone();
                    let corpse_position = corpse.position;
                    
                    // Drop the immutable borrow before performing the action
                    let _ = floor;
                    
                    self.perform_corpse_action(dungeon_state, action.clone(), &corpse_name, corpse_position)?;
                } else {
                    self.add_dungeon_message(dungeon_state, 
                        format!("Invalid action number. Available actions: 1-{}", corpse.interactions.len()));
                }
            } else {
                self.add_dungeon_message(dungeon_state, "You are not standing on a corpse.".to_string());
            }
        }
        
        Ok(())
    }
    
    fn perform_corpse_action(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, action: crate::world::CorpseInteraction, corpse_name: &str, corpse_position: crate::world::LocalCoord) -> anyhow::Result<()> {
        match action {
            crate::world::CorpseInteraction::Loot => {
                self.auto_loot_corpse(dungeon_state, corpse_position)?;
            }
            crate::world::CorpseInteraction::Examine => {
                if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
                    if let Some(corpse) = floor.corpses.iter().find(|c| c.position == corpse_position) {
                        let decay_description = corpse.get_decay_description();
                        let decay_level = corpse.decay_level;
                        let loot_generated = corpse.loot_generated;
                        
                        self.add_dungeon_message(dungeon_state, decay_description);
                        self.add_dungeon_message(dungeon_state, 
                            format!("Decay level: {}/10", decay_level));
                        
                        if loot_generated {
                            self.add_dungeon_message(dungeon_state, "This corpse has already been looted.".to_string());
                        }
                    }
                }
            }
            crate::world::CorpseInteraction::Skin => {
                self.add_dungeon_message(dungeon_state, format!("You carefully skin the {}, obtaining hide and meat.", corpse_name));
                if let Some(character) = &mut self.current_character {
                    let hide_item = crate::forge::InventoryItem {
                        name: "Animal Hide".to_string(),
                        item_type: crate::forge::ItemType::Material(crate::forge::MaterialItem {
                            material_type: crate::forge::MaterialType::Leather,
                            quality: crate::forge::Quality::Common,
                        }),
                        weight: 2.0,
                        stack_size: 10,
                        quantity: 1,
                        value: 5,
                        description: "Raw hide from an animal. Can be used for crafting.".to_string(),
                    };
                    let meat_item = crate::forge::InventoryItem {
                        name: "Raw Meat".to_string(),
                        item_type: crate::forge::ItemType::Consumable(crate::forge::ConsumableItem {
                            consumable_type: crate::forge::ConsumableType::Food,
                            effect: "Restores hunger when cooked".to_string(),
                            duration: None,
                        }),
                        weight: 1.0,
                        stack_size: 20,
                        quantity: 1,
                        value: 1,
                        description: "Fresh meat from an animal. Needs cooking before eating.".to_string(),
                    };
                    character.inventory.add_item(hide_item).ok();
                    character.inventory.add_item(meat_item).ok();
                }
            }
            crate::world::CorpseInteraction::Harvest => {
                self.add_dungeon_message(dungeon_state, format!("You harvest magical components from the {}.", corpse_name));
                if let Some(character) = &mut self.current_character {
                    let component_item = crate::forge::InventoryItem {
                        name: "Spell Component".to_string(),
                        item_type: crate::forge::ItemType::Consumable(crate::forge::ConsumableItem {
                            consumable_type: crate::forge::ConsumableType::Reagent,
                            effect: "Used in spell casting".to_string(),
                            duration: None,
                        }),
                        weight: 0.1,
                        stack_size: 50,
                        quantity: 1,
                        value: 10,
                        description: "A magical component harvested from a creature.".to_string(),
                    };
                    character.inventory.add_item(component_item).ok();
                }
            }
            crate::world::CorpseInteraction::RaiseSkeleton => {
                self.add_dungeon_message(dungeon_state, format!("You attempt to raise the {} as a skeleton...", corpse_name));
                self.add_dungeon_message(dungeon_state, "The necromantic energies swirl, but nothing happens. (Spell not yet implemented)".to_string());
            }
            crate::world::CorpseInteraction::RaiseZombie => {
                self.add_dungeon_message(dungeon_state, format!("You attempt to raise the {} as a zombie...", corpse_name));
                self.add_dungeon_message(dungeon_state, "The necromantic energies swirl, but nothing happens. (Spell not yet implemented)".to_string());
            }
            crate::world::CorpseInteraction::Burn => {
                self.add_dungeon_message(dungeon_state, format!("You set fire to the corpse of {}.", corpse_name));
                self.add_dungeon_message(dungeon_state, "The corpse burns away, leaving only ash.".to_string());
                
                // Remove the corpse from the floor
                if let Some(floor) = dungeon_state.dungeon.get_current_floor_mut() {
                    floor.corpses.retain(|c| c.position != corpse_position);
                }
            }
        }
        
        Ok(())
    }

    fn add_dungeon_message(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, message: String) {
        dungeon_state.messages.push(message);
        // Keep only the last 20 messages to prevent memory growth
        if dungeon_state.messages.len() > 20 {
            dungeon_state.messages.remove(0);
        }
    }

    fn find_nearby_pois(&mut self, world_state: &mut WorldExplorationState) -> anyhow::Result<()> {
        let player_pos = world_state.player_local_pos;
        let mut messages = Vec::new();
        
        if let Some(zone_data) = &world_state.zone_data {
            messages.push("=== NEARBY POINTS OF INTEREST ===".to_string());
            
            let mut pois_found = false;
            
            // Search in expanding radius
            for radius in 1..=10 {
                let pois_at_radius: Vec<&crate::world::PointOfInterest> = zone_data.points_of_interest.iter()
                    .filter(|poi| {
                        let dx = (poi.position.x - player_pos.x).abs();
                        let dy = (poi.position.y - player_pos.y).abs();
                        let distance = ((dx * dx + dy * dy) as f32).sqrt() as i32;
                        distance == radius
                    })
                    .collect();
                
                if !pois_at_radius.is_empty() {
                    for poi in pois_at_radius {
                        let dx = poi.position.x - player_pos.x;
                        let dy = poi.position.y - player_pos.y;
                        let direction = if dx == 0 && dy < 0 { "North" }
                                      else if dx > 0 && dy < 0 { "Northeast" }
                                      else if dx > 0 && dy == 0 { "East" }
                                      else if dx > 0 && dy > 0 { "Southeast" }
                                      else if dx == 0 && dy > 0 { "South" }
                                      else if dx < 0 && dy > 0 { "Southwest" }
                                      else if dx < 0 && dy == 0 { "West" }
                                      else if dx < 0 && dy < 0 { "Northwest" }
                                      else { "Here" };
                        
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        let can_enter = self.can_enter_poi(&poi.poi_type);
                        let enter_text = if can_enter { " [ENTERABLE]" } else { "" };
                        
                        messages.push(format!("📍 {} - {} ({:.1} tiles){}", 
                            poi.name, direction, distance, enter_text));
                        pois_found = true;
                    }
                }
            }
            
            if !pois_found {
                messages.push("No points of interest found in this area.".to_string());
                messages.push("Try exploring different zones or moving around.".to_string());
            } else {
                messages.push("".to_string());
                messages.push("Move close to an [ENTERABLE] location and press 'E' to explore inside!".to_string());
            }
        } else {
            messages.push("Zone data not loaded. Cannot search for points of interest.".to_string());
        }
        
        for message in messages {
            self.add_message(world_state, message);
        }
        
        Ok(())
    }

    fn initiate_dungeon_combat(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        
        // Find creatures within attack range (adjacent tiles or same tile)
        let nearby_creatures: Vec<crate::world::DungeonCreature> = if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            floor.creatures.iter()
                .filter(|creature| {
                    let dx = (creature.position.x - player_pos.x).abs();
                    let dy = (creature.position.y - player_pos.y).abs();
                    // Allow combat with creatures on same tile or adjacent tiles
                    dx <= 1 && dy <= 1
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        
        if !nearby_creatures.is_empty() {
            // Attack the first nearby creature
            let target_creature = &nearby_creatures[0];
            self.add_dungeon_message(dungeon_state, format!("⚔️ Engaging {} in combat!", target_creature.name));
            self.start_dungeon_combat(dungeon_state, target_creature)?;
        } else {
            // Check if there are any creatures on the floor at all for debugging
            let (has_creatures, creature_info) = if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
                if floor.creatures.is_empty() {
                    (false, Vec::new())
                } else {
                    let info: Vec<String> = floor.creatures.iter()
                        .map(|creature| {
                            let dx = (creature.position.x - player_pos.x).abs();
                            let dy = (creature.position.y - player_pos.y).abs();
                            format!("  {} at ({}, {}) - distance: {}", 
                                creature.name, creature.position.x, creature.position.y, dx + dy)
                        })
                        .collect();
                    (true, info)
                }
            } else {
                (false, Vec::new())
            };
            
            if !has_creatures {
                self.add_dungeon_message(dungeon_state, "🔍 No creatures on this floor. Starting random encounter...".to_string());
            } else {
                self.add_dungeon_message(dungeon_state, "🔍 No creatures within range. Move closer to attack, or starting random encounter...".to_string());
                // Debug: show creature positions
                for info in creature_info {
                    self.add_dungeon_message(dungeon_state, info);
                }
            }
            
            // Start a random encounter using Forge rules
            if let Some(character) = &self.current_character {
                let character = character.clone();
                self.start_dungeon_random_encounter(&character, dungeon_state)?;
            }
        }
        
        Ok(())
    }

    fn initiate_ranged_combat(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        let player_pos = dungeon_state.player_pos;
        
        // Get the player's vision radius to determine ranged attack range
        let vision_radius = if let Some(character) = &self.current_character {
            character.get_vision_radius() as i32
        } else {
            3 // Default fallback
        };
        
        // Find creatures within vision range (but exclude adjacent ones for ranged preference)
        let ranged_creatures: Vec<crate::world::DungeonCreature> = if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
            floor.creatures.iter()
                .filter(|creature| {
                    let dx = (creature.position.x - player_pos.x).abs();
                    let dy = (creature.position.y - player_pos.y).abs();
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    
                    // Only visible creatures within vision range but further than adjacent
                    distance > 1.5 && distance <= vision_radius as f32
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        
        if !ranged_creatures.is_empty() {
            // Attack the first visible creature at range
            let target_creature = &ranged_creatures[0];
            let distance = {
                let dx = (target_creature.position.x - player_pos.x).abs();
                let dy = (target_creature.position.y - player_pos.y).abs();
                ((dx * dx + dy * dy) as f32).sqrt()
            };
            
            self.add_dungeon_message(dungeon_state, 
                format!("🏹 Targeting {} at range! (distance: {:.1} tiles)", target_creature.name, distance));
            self.add_dungeon_message(dungeon_state, 
                "💥 You get the drop on them with a ranged attack!".to_string());
            
            // Start combat with ranged advantage - player gets to act first
            self.start_ranged_dungeon_combat(dungeon_state, target_creature)?;
        } else {
            // Check for any visible creatures at all
            let visible_creatures: Vec<crate::world::DungeonCreature> = if let Some(floor) = dungeon_state.dungeon.get_current_floor() {
                floor.creatures.iter()
                    .filter(|creature| {
                        let dx = (creature.position.x - player_pos.x).abs();
                        let dy = (creature.position.y - player_pos.y).abs();
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        distance <= vision_radius as f32
                    })
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            
            if visible_creatures.is_empty() {
                self.add_dungeon_message(dungeon_state, "🔍 No creatures visible for ranged attack.".to_string());
            } else {
                self.add_dungeon_message(dungeon_state, "🏹 No creatures at ranged distance. Use F for melee combat.".to_string());
            }
        }
        
        Ok(())
    }

    fn start_dungeon_combat(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, target_creature: &crate::world::DungeonCreature) -> anyhow::Result<()> {
        if let Some(character) = &self.current_character {
            // Create player tactical combatant
            let player_participant = self.create_player_combat_participant(character)?;
            let player_tactical = crate::forge::TacticalCombatParticipant {
                base_participant: player_participant,
                position: crate::forge::BattlefieldPosition::new(5, 10), // Start player at left side of larger battlefield
                movement_capabilities: crate::forge::MovementCapabilities::default(),
                movement_remaining: 3,
                has_acted: false,
                declared_action: None,
            };
            
            // Create creature tactical combatant
            let creature_participant = self.create_creature_combat_participant(target_creature);
            let creature_tactical = crate::forge::TacticalCombatParticipant {
                base_participant: creature_participant,
                position: crate::forge::BattlefieldPosition::new(25, 10), // Start creature on right side of larger battlefield
                movement_capabilities: crate::forge::MovementCapabilities {
                    movement_speed: 2,
                    can_fly: false,
                    can_swim: false,
                    can_climb: false,
                },
                movement_remaining: 2,
                has_acted: false,
                declared_action: None,
            };
            
            let tactical_participants = vec![player_tactical, creature_tactical];
            
            // Create dungeon battlefield - large enough for tactical maneuvering
            let mut battlefield = crate::forge::TacticalBattlefield::generate_battlefield(35, 20, crate::forge::BattlefieldComplexity::Moderate);
            
            // Place participants on battlefield
            for (i, participant) in tactical_participants.iter().enumerate() {
                battlefield.participant_positions.insert(i, participant.position);
            }
            
            // Create tactical combat state with return path to dungeon
            let tactical_combat_state = crate::ui::TacticalCombatState::new(
                battlefield,
                tactical_participants,
                Some(dungeon_state.clone()), // Store dungeon state to return after combat
            );

            // DON'T auto-process AI turns - let player see initiative and control flow
            // self.process_ai_turns_until_player(&mut tactical_combat_state)?;

            // Transition to standalone tactical combat
            self.state = UIState::TacticalCombat(tactical_combat_state);
        }

        Ok(())
    }

    fn start_ranged_dungeon_combat(&mut self, dungeon_state: &mut crate::ui::DungeonExplorationState, target_creature: &crate::world::DungeonCreature) -> anyhow::Result<()> {
        if let Some(character) = &self.current_character {
            // Create player tactical combatant
            let player_participant = self.create_player_combat_participant(character)?;
            let player_tactical = crate::forge::TacticalCombatParticipant {
                base_participant: player_participant,
                position: crate::forge::BattlefieldPosition::new(5, 11), // Start player at left side of larger battlefield
                movement_capabilities: crate::forge::MovementCapabilities::default(),
                movement_remaining: 4, // Extra movement for ranged advantage
                has_acted: false,
                declared_action: None,
            };
            
            // Create creature tactical combatant (starts farther away for ranged)
            let creature_participant = self.create_creature_combat_participant(target_creature);
            let creature_tactical = crate::forge::TacticalCombatParticipant {
                base_participant: creature_participant,
                position: crate::forge::BattlefieldPosition::new(35, 11), // Start creature much farther away for ranged combat
                movement_capabilities: crate::forge::MovementCapabilities {
                    movement_speed: 2,
                    can_fly: false,
                    can_swim: false,
                    can_climb: false,
                },
                movement_remaining: 2,
                has_acted: false,
                declared_action: None,
            };
            
            let tactical_participants = vec![player_tactical, creature_tactical];
            
            // Create dungeon battlefield with more open terrain for ranged combat
            let mut battlefield = crate::forge::TacticalBattlefield::generate_battlefield(45, 22, crate::forge::BattlefieldComplexity::Simple);
            
            // Place participants on battlefield
            for (i, participant) in tactical_participants.iter().enumerate() {
                battlefield.participant_positions.insert(i, participant.position);
            }
            
            // Create tactical combat state with return path to dungeon
            let mut tactical_combat_state = crate::ui::TacticalCombatState::new(
                battlefield,
                tactical_participants,
                Some(dungeon_state.clone()), // Store dungeon state to return after combat
            );

            // Add ranged advantage message to combat log
            tactical_combat_state.combat_log.push("🏹 You struck first with a ranged attack!".to_string());
            tactical_combat_state.combat_log.push("🎯 You have tactical advantage and extra movement!".to_string());

            // DON'T auto-process AI turns - let player see initiative and control flow
            // self.process_ai_turns_until_player(&mut tactical_combat_state)?;

            // Transition to standalone tactical combat (stores dungeon state in return_to_dungeon)
            self.state = UIState::TacticalCombat(tactical_combat_state);
        }

        Ok(())
    }

    fn start_dungeon_random_encounter(&mut self, character: &ForgeCharacter, _dungeon_state: &crate::ui::DungeonExplorationState) -> anyhow::Result<()> {
        // Create player tactical combatant
        let mut player_participant = CombatParticipant::from_character(character, Some(Weapon::rusty_sword()));
        player_participant.armor = Some(Armor::leather());
        
        let player_tactical = crate::forge::TacticalCombatParticipant {
            base_participant: player_participant,
            position: crate::forge::BattlefieldPosition::new(5, 12), // Start player at left side of larger battlefield
            movement_capabilities: crate::forge::MovementCapabilities::default(),
            movement_remaining: 3,
            has_acted: false,
            declared_action: None,
        };
        
        // Generate random dungeon enemies
        let enemy_participants = self.generate_dungeon_enemies()?;
        
        // Convert enemies to tactical participants
        let mut tactical_participants = vec![player_tactical];
        for (i, enemy) in enemy_participants.into_iter().enumerate() {
            let enemy_tactical = crate::forge::TacticalCombatParticipant {
                base_participant: enemy,
                position: crate::forge::BattlefieldPosition::new(25 + (i as i32) * 3, 10 + (i as i32) * 2), // Spread enemies on right side of larger battlefield
                movement_capabilities: crate::forge::MovementCapabilities {
                    movement_speed: 2,
                    can_fly: false,
                    can_swim: false,
                    can_climb: false,
                },
                movement_remaining: 2,
                has_acted: false,
                declared_action: None,
            };
            tactical_participants.push(enemy_tactical);
        }
        
        // Create dungeon battlefield with high complexity for random encounters
        let mut battlefield = crate::forge::TacticalBattlefield::generate_battlefield(38, 24, crate::forge::BattlefieldComplexity::Complex);
        
        // Place participants on battlefield
        for (i, participant) in tactical_participants.iter().enumerate() {
            battlefield.participant_positions.insert(i, participant.position);
        }
        
        // Create tactical combat state
        let mut tactical_combat_state = crate::ui::TacticalCombatState::new(
            battlefield,
            tactical_participants,
            None, // No need for return_to_dungeon since we're integrated
        );
        
        // Add random encounter message
        tactical_combat_state.combat_log.push("⚔️ Random encounter! Multiple enemies appear!".to_string());

        // DON'T auto-process AI turns - let player see initiative popup and control flow
        // self.process_ai_turns_until_player(&mut tactical_combat_state)?;

        // Transition to standalone tactical combat (stores dungeon state in return_to_dungeon)
        self.state = UIState::TacticalCombat(tactical_combat_state);

        Ok(())
    }

    fn generate_dungeon_enemies(&self) -> anyhow::Result<Vec<CombatParticipant>> {
        let mut rng = rand::thread_rng();
        let mut enemies = Vec::new();
        
        // Generate enemies typical for dungeon environments
        match rng.gen_range(0..10) {
            0..=2 => enemies.push(create_skeleton()),
            3..=4 => enemies.push(create_zombie()),
            5..=6 => enemies.push(create_goblin()),
            7..=8 => enemies.push(create_giant_spider()),
            _ => {
                // Multiple enemies
                enemies.push(create_skeleton());
                if rng.gen_bool(0.5) {
                    enemies.push(create_skeleton());
                }
            }
        }
        
        Ok(enemies)
    }

    fn create_player_combat_participant(&self, character: &ForgeCharacter) -> anyhow::Result<CombatParticipant> {
        Ok(CombatParticipant {
            name: character.name.clone(),
            combat_stats: character.combat_stats.clone(),
            weapon: Some(Weapon::unarmed()), // TODO: Get actual equipped weapon
            armor: None, // TODO: Get actual equipped armor
            shield: None, // TODO: Get actual equipped shield
            initiative: 0, // Will be rolled
            is_player: true,
            ai_personality: None, // Players don't have AI personalities
            magic: character.magic.clone(),
        })
    }

    fn create_creature_combat_participant(&self, creature: &crate::world::DungeonCreature) -> CombatParticipant {
        // Convert dungeon creature to combat participant with Forge-based stats
        let (stats, weapon) = match creature.creature_type {
            crate::world::CreatureType::Rat => {
                (self.create_rat_stats(), Some(self.create_rat_bite()))
            },
            crate::world::CreatureType::Bat => {
                (self.create_bat_stats(), Some(self.create_bat_bite()))
            },
            crate::world::CreatureType::Spider => {
                (self.create_spider_stats(), Some(self.create_spider_bite()))
            },
            crate::world::CreatureType::Skeleton => {
                (self.create_skeleton_stats(), Some(Weapon::rusty_sword()))
            },
            crate::world::CreatureType::Zombie => {
                (self.create_zombie_stats(), Some(self.create_zombie_claws()))
            },
            crate::world::CreatureType::Goblin => {
                (self.create_goblin_stats(), Some(self.create_goblin_spear()))
            },
            _ => {
                // Default creature stats
                (self.create_default_creature_stats(), Some(Weapon::unarmed()))
            }
        };
        
        CombatParticipant {
            name: creature.name.clone(),
            combat_stats: stats,
            weapon,
            armor: None,
            shield: None,
            initiative: 0, // Will be rolled
            is_player: false,
            ai_personality: Some(crate::forge::AIPersonality::default()), // Default AI for dungeon creatures
            magic: crate::forge::magic::MagicSystem::new(3), // Basic magic for creatures
        }
    }

    #[allow(dead_code)]
    fn get_player_skills(&self, character: &ForgeCharacter) -> Vec<String> {
        let mut skills = vec!["Melee Combat".to_string()]; // Everyone has basic melee
        
        // Add skills from character
        for (skill_name, skill_level) in &character.skills {
            if *skill_level > 0 {
                skills.push(skill_name.clone());
            }
        }
        
        // Add racial abilities
        for ability in &character.race.special_abilities {
            skills.push(ability.clone());
        }
        
        skills
    }


    // Creature stat creation functions based on Forge rules
    fn create_rat_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 2, max: 2 },
            attack_value: 8, // Low attack
            defensive_value: 12, // Quick and dodgy
            damage_bonus: -2, // Weak
        }
    }

    fn create_bat_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 3, max: 3 },
            attack_value: 10, // Flying gives bonus
            defensive_value: 14, // Very hard to hit
            damage_bonus: -1,
        }
    }

    fn create_spider_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 4, max: 4 },
            attack_value: 11, // Venomous bite
            defensive_value: 13, // Quick
            damage_bonus: 0,
        }
    }

    fn create_skeleton_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 8, max: 8 },
            attack_value: 12, // Armed with sword
            defensive_value: 11, // Bone armor
            damage_bonus: 1,
        }
    }

    fn create_zombie_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 12, max: 12 },
            attack_value: 10, // Slow but strong
            defensive_value: 9, // Slow and clumsy
            damage_bonus: 2,
        }
    }

    fn create_goblin_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 6, max: 6 },
            attack_value: 11, // Armed and trained
            defensive_value: 12, // Small and quick
            damage_bonus: 0,
        }
    }

    fn create_default_creature_stats(&self) -> crate::forge::CombatStats {
        use crate::forge::{CombatStats, HealthPoints};
        CombatStats {
            hit_points: HealthPoints { current: 8, max: 8 },
            attack_value: 10,
            defensive_value: 10,
            damage_bonus: 0,
        }
    }

    // Creature weapon creation functions
    fn create_rat_bite(&self) -> Weapon {
        use crate::forge::{DamageType, WeaponType};
        Weapon {
            name: "Bite".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d2".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    fn create_bat_bite(&self) -> Weapon {
        use crate::forge::{DamageType, WeaponType};
        Weapon {
            name: "Bite".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d3".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 2, // Flying bonus
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    fn create_spider_bite(&self) -> Weapon {
        use crate::forge::{DamageType, WeaponType};
        Weapon {
            name: "Venomous Bite".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d4".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 1,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    fn create_zombie_claws(&self) -> Weapon {
        use crate::forge::{DamageType, WeaponType};
        Weapon {
            name: "Claws".to_string(),
            weapon_type: WeaponType::Unarmed,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Slashing,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    fn create_goblin_spear(&self) -> Weapon {
        use crate::forge::{DamageType, WeaponType};
        Weapon {
            name: "Crude Spear".to_string(),
            weapon_type: WeaponType::Spear,
            damage_dice: "1d6".to_string(),
            damage_type: DamageType::Piercing,
            damage_bonus: 0,
            attack_bonus: 0,
            two_handed: false,
            ranged: false,
            range: None,
        }
    }

    fn handle_inventory_input(&mut self, key: KeyEvent, mut inventory_state: crate::ui::InventoryState) -> anyhow::Result<()> {
        if let Some(character) = &mut self.current_character {
            // Ensure sorted indices are up to date
            inventory_state.sorted_indices = inventory_state.compute_sorted_indices(&character.inventory.items);
            
            // Helper to get original index from display index
            let get_original_index = |display_index: usize| -> Option<usize> {
                inventory_state.sorted_indices.get(display_index).copied()
            };
            
            match key.code {
                KeyCode::Esc => {
                    self.state = UIState::CharacterMenu;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if inventory_state.selected_index > 0 {
                        inventory_state.selected_index -= 1;
                    }
                    self.state = UIState::InventoryManagement(inventory_state);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max_display_index = inventory_state.sorted_indices.len().saturating_sub(1);
                    if inventory_state.selected_index < max_display_index {
                        inventory_state.selected_index += 1;
                    }
                    self.state = UIState::InventoryManagement(inventory_state);
                }
                KeyCode::Enter => {
                    // Use/Equip item - get original index from display index
                    if let Some(original_index) = get_original_index(inventory_state.selected_index) {
                        if let Some(item) = character.inventory.items.get(original_index) {
                        match &item.item_type {
                            crate::forge::ItemType::Weapon(weapon) => {
                                // Equip weapon
                                let old_weapon = character.equipment.weapon.take();
                                character.equipment.weapon = Some(weapon.clone());
                                
                                // Add old weapon back to inventory if there was one
                                if let Some(old) = old_weapon {
                                    let name = old.name.clone();
                                    let inventory_item = crate::forge::InventoryItem {
                                        item_type: crate::forge::ItemType::Weapon(old),
                                        name: name.clone(),
                                        weight: 3.0, // Default weapon weight
                                        stack_size: 1,
                                        quantity: 1,
                                        value: 25,
                                        description: format!("A {}", name.to_lowercase()),
                                    };
                                    let _ = character.inventory.add_item(inventory_item);
                                }
                                
                                // Remove equipped item from inventory using original index
                                character.inventory.items.remove(original_index);
                                // Adjust selected index if necessary (display index remains, but total count decreases)
                                let new_max = character.inventory.items.len().saturating_sub(1);
                                if inventory_state.selected_index > new_max && new_max > 0 {
                                    inventory_state.selected_index = new_max;
                                } else if character.inventory.items.is_empty() {
                                    inventory_state.selected_index = 0;
                                }
                            }
                            crate::forge::ItemType::Armor(armor) => {
                                // Determine if this is shield or armor
                                if armor.name.contains("Shield") {
                                    let old_shield = character.equipment.shield.take();
                                    character.equipment.shield = Some(armor.clone());
                                    
                                    if let Some(old) = old_shield {
                                        let name = old.name.clone();
                                        let inventory_item = crate::forge::InventoryItem {
                                            item_type: crate::forge::ItemType::Armor(old),
                                            name: name.clone(),
                                            weight: 4.0,
                                            stack_size: 1,
                                            quantity: 1,
                                            value: 30,
                                            description: format!("A {}", name.to_lowercase()),
                                        };
                                        let _ = character.inventory.add_item(inventory_item);
                                    }
                                } else {
                                    let old_armor = character.equipment.armor.take();
                                    character.equipment.armor = Some(armor.clone());
                                    
                                    if let Some(old) = old_armor {
                                        let name = old.name.clone();
                                        let inventory_item = crate::forge::InventoryItem {
                                            item_type: crate::forge::ItemType::Armor(old),
                                            name: name.clone(),
                                            weight: 8.0,
                                            stack_size: 1,
                                            quantity: 1,
                                            value: 40,
                                            description: format!("A {}", name.to_lowercase()),
                                        };
                                        let _ = character.inventory.add_item(inventory_item);
                                    }
                                }
                                
                                character.inventory.items.remove(original_index);
                                let new_max = character.inventory.items.len().saturating_sub(1);
                                if inventory_state.selected_index > new_max && new_max > 0 {
                                    inventory_state.selected_index = new_max;
                                } else if character.inventory.items.is_empty() {
                                    inventory_state.selected_index = 0;
                                }
                            }
                            crate::forge::ItemType::Accessory(accessory) => {
                                // Try to equip in first available accessory slot
                                if character.equipment.accessory1.is_none() {
                                    character.equipment.accessory1 = Some(accessory.clone());
                                } else if character.equipment.accessory2.is_none() {
                                    character.equipment.accessory2 = Some(accessory.clone());
                                } else {
                                    // Both slots full, replace first one
                                    let old_accessory = character.equipment.accessory1.take();
                                    character.equipment.accessory1 = Some(accessory.clone());
                                    
                                    if let Some(old) = old_accessory {
                                        let name = old.name.clone();
                                        let inventory_item = crate::forge::InventoryItem {
                                            item_type: crate::forge::ItemType::Accessory(old),
                                            name: name.clone(),
                                            weight: 0.5,
                                            stack_size: 1,
                                            quantity: 1,
                                            value: 50,
                                            description: format!("A {}", name.to_lowercase()),
                                        };
                                        let _ = character.inventory.add_item(inventory_item);
                                    }
                                }
                                
                                character.inventory.items.remove(original_index);
                                let new_max = character.inventory.items.len().saturating_sub(1);
                                if inventory_state.selected_index > new_max && new_max > 0 {
                                    inventory_state.selected_index = new_max;
                                } else if character.inventory.items.is_empty() {
                                    inventory_state.selected_index = 0;
                                }
                            }
                            crate::forge::ItemType::Consumable(_) => {
                                // Use consumable (basic implementation)
                                // For now, just remove one from inventory
                                if let Some(item) = character.inventory.items.get_mut(original_index) {
                                    if item.quantity > 1 {
                                        item.quantity -= 1;
                                    } else {
                                        character.inventory.items.remove(original_index);
                                        let new_max = character.inventory.items.len().saturating_sub(1);
                                        if inventory_state.selected_index > new_max && new_max > 0 {
                                            inventory_state.selected_index = new_max;
                                        } else if character.inventory.items.is_empty() {
                                            inventory_state.selected_index = 0;
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Other item types - no action for now
                            }
                        }
                        
                        // Equipment change complete
                        }
                    }
                    self.state = UIState::InventoryManagement(inventory_state);
                }
                KeyCode::Char('d') => {
                    // Drop item - get original index from display index
                    if !character.inventory.items.is_empty() && inventory_state.selected_index < inventory_state.sorted_indices.len() {
                        if let Some(original_index) = get_original_index(inventory_state.selected_index) {
                            character.inventory.items.remove(original_index);
                            let new_max = character.inventory.items.len().saturating_sub(1);
                            if inventory_state.selected_index > new_max && new_max > 0 {
                                inventory_state.selected_index = new_max;
                            } else if character.inventory.items.is_empty() {
                                inventory_state.selected_index = 0;
                            }
                        }
                    }
                    self.state = UIState::InventoryManagement(inventory_state);
                }
                KeyCode::Tab => {
                    // Cycle sort mode
                    inventory_state.sort_mode = match inventory_state.sort_mode {
                        crate::ui::InventorySortMode::Name => crate::ui::InventorySortMode::Type,
                        crate::ui::InventorySortMode::Type => crate::ui::InventorySortMode::Weight,
                        crate::ui::InventorySortMode::Weight => crate::ui::InventorySortMode::Value,
                        crate::ui::InventorySortMode::Value => crate::ui::InventorySortMode::Quantity,
                        crate::ui::InventorySortMode::Quantity => crate::ui::InventorySortMode::Name,
                    };
                    self.state = UIState::InventoryManagement(inventory_state);
                }
                _ => {
                    self.state = UIState::InventoryManagement(inventory_state);
                }
            }
        }
        Ok(())
    }

    fn handle_equipment_input(&mut self, key: KeyEvent, mut equipment_state: crate::ui::EquipmentState) -> anyhow::Result<()> {
        if let Some(character) = &mut self.current_character {
            match key.code {
                KeyCode::Esc => {
                    self.state = UIState::CharacterMenu;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    // Navigate equipment slots
                    equipment_state.selected_slot = match equipment_state.selected_slot {
                        crate::ui::EquipmentSlot::Weapon => crate::ui::EquipmentSlot::Accessory2,
                        crate::ui::EquipmentSlot::Armor => crate::ui::EquipmentSlot::Weapon,
                        crate::ui::EquipmentSlot::Shield => crate::ui::EquipmentSlot::Armor,
                        crate::ui::EquipmentSlot::Accessory1 => crate::ui::EquipmentSlot::Shield,
                        crate::ui::EquipmentSlot::Accessory2 => crate::ui::EquipmentSlot::Accessory1,
                    };
                    let compatible_items = Self::get_compatible_items_static(character, &equipment_state.selected_slot);
                    equipment_state.available_items = compatible_items;
                    equipment_state.selected_item_index = 0;
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Navigate equipment slots
                    equipment_state.selected_slot = match equipment_state.selected_slot {
                        crate::ui::EquipmentSlot::Weapon => crate::ui::EquipmentSlot::Armor,
                        crate::ui::EquipmentSlot::Armor => crate::ui::EquipmentSlot::Shield,
                        crate::ui::EquipmentSlot::Shield => crate::ui::EquipmentSlot::Accessory1,
                        crate::ui::EquipmentSlot::Accessory1 => crate::ui::EquipmentSlot::Accessory2,
                        crate::ui::EquipmentSlot::Accessory2 => crate::ui::EquipmentSlot::Weapon,
                    };
                    let compatible_items = Self::get_compatible_items_static(character, &equipment_state.selected_slot);
                    equipment_state.available_items = compatible_items;
                    equipment_state.selected_item_index = 0;
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    // Navigate available items
                    if equipment_state.selected_item_index > 0 {
                        equipment_state.selected_item_index -= 1;
                    }
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Navigate available items
                    if equipment_state.selected_item_index < equipment_state.available_items.len().saturating_sub(1) {
                        equipment_state.selected_item_index += 1;
                    }
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
                KeyCode::Enter => {
                    // Equip selected item
                    if let Some(_item) = equipment_state.available_items.get(equipment_state.selected_item_index) {
                        // Implementation similar to inventory equip logic
                        // This is a simplified version - full implementation would handle slot-specific logic
                    }
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
                KeyCode::Char('u') => {
                    // Unequip item from selected slot
                    match equipment_state.selected_slot {
                        crate::ui::EquipmentSlot::Weapon => {
                            if let Some(weapon) = character.equipment.weapon.take() {
                                let name = weapon.name.clone();
                                let inventory_item = crate::forge::InventoryItem {
                                    item_type: crate::forge::ItemType::Weapon(weapon),
                                    name: name.clone(),
                                    weight: 3.0,
                                    stack_size: 1,
                                    quantity: 1,
                                    value: 25,
                                    description: format!("A {}", name.to_lowercase()),
                                };
                                let _ = character.inventory.add_item(inventory_item);
                            }
                        }
                        crate::ui::EquipmentSlot::Armor => {
                            if let Some(armor) = character.equipment.armor.take() {
                                let name = armor.name.clone();
                                let inventory_item = crate::forge::InventoryItem {
                                    item_type: crate::forge::ItemType::Armor(armor),
                                    name: name.clone(),
                                    weight: 8.0,
                                    stack_size: 1,
                                    quantity: 1,
                                    value: 40,
                                    description: format!("A {}", name.to_lowercase()),
                                };
                                let _ = character.inventory.add_item(inventory_item);
                            }
                        }
                        crate::ui::EquipmentSlot::Shield => {
                            if let Some(shield) = character.equipment.shield.take() {
                                let name = shield.name.clone();
                                let inventory_item = crate::forge::InventoryItem {
                                    item_type: crate::forge::ItemType::Armor(shield),
                                    name: name.clone(),
                                    weight: 4.0,
                                    stack_size: 1,
                                    quantity: 1,
                                    value: 30,
                                    description: format!("A {}", name.to_lowercase()),
                                };
                                let _ = character.inventory.add_item(inventory_item);
                            }
                        }
                        crate::ui::EquipmentSlot::Accessory1 => {
                            if let Some(accessory) = character.equipment.accessory1.take() {
                                let name = accessory.name.clone();
                                let inventory_item = crate::forge::InventoryItem {
                                    item_type: crate::forge::ItemType::Accessory(accessory),
                                    name: name.clone(),
                                    weight: 0.5,
                                    stack_size: 1,
                                    quantity: 1,
                                    value: 50,
                                    description: format!("A {}", name.to_lowercase()),
                                };
                                let _ = character.inventory.add_item(inventory_item);
                            }
                        }
                        crate::ui::EquipmentSlot::Accessory2 => {
                            if let Some(accessory) = character.equipment.accessory2.take() {
                                let name = accessory.name.clone();
                                let inventory_item = crate::forge::InventoryItem {
                                    item_type: crate::forge::ItemType::Accessory(accessory),
                                    name: name.clone(),
                                    weight: 0.5,
                                    stack_size: 1,
                                    quantity: 1,
                                    value: 50,
                                    description: format!("A {}", name.to_lowercase()),
                                };
                                let _ = character.inventory.add_item(inventory_item);
                            }
                        }
                    }
                    
                    // Equipment change complete
                    
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
                _ => {
                    self.state = UIState::EquipmentManagement(equipment_state);
                }
            }
        }
        Ok(())
    }

    fn handle_tactical_combat_input(&mut self, key: KeyEvent, mut tactical_state: crate::ui::TacticalCombatState) -> anyhow::Result<()> {
        match key.code {
            // Arrow keys - move cursor
            KeyCode::Up => {
                tactical_state.cursor_pos.1 = (tactical_state.cursor_pos.1 - 1).max(0);
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Down => {
                tactical_state.cursor_pos.1 = (tactical_state.cursor_pos.1 + 1).min(tactical_state.battlefield.height as i32 - 1);
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Left => {
                tactical_state.cursor_pos.0 = (tactical_state.cursor_pos.0 - 1).max(0);
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Right => {
                tactical_state.cursor_pos.0 = (tactical_state.cursor_pos.0 + 1).min(tactical_state.battlefield.width as i32 - 1);
                self.state = UIState::TacticalCombat(tactical_state);
            }

            // Action hotkeys
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Attack action
                tactical_state.add_log("Selected Attack action. Use arrow keys to target an enemy.".to_string());
                tactical_state.pending_action = Some(crate::ui::PendingAction::Attack { target_index: None });
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                // Move action
                tactical_state.add_log("Selected Move action. Use arrow keys to select destination.".to_string());
                tactical_state.pending_action = Some(crate::ui::PendingAction::Move { destination: None });
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Cast spell action
                tactical_state.add_log("Cast spell - not yet implemented.".to_string());
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                // Use item action
                tactical_state.add_log("Use item - not yet implemented.".to_string());
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Defend action
                tactical_state.add_log("Taking defensive stance...".to_string());
                tactical_state.pending_action = Some(crate::ui::PendingAction::Defend);
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Skills action
                tactical_state.add_log("Skills - not yet implemented.".to_string());
                self.state = UIState::TacticalCombat(tactical_state);
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // End turn
                tactical_state.add_log("Ending turn...".to_string());
                tactical_state.pending_action = None;
                self.advance_turn(&mut tactical_state);
                self.state = UIState::TacticalCombat(tactical_state);
            }

            // Enter key - confirm selection or execute command
            KeyCode::Enter => {
                if !tactical_state.command_buffer.is_empty() {
                    // Execute MUD-style command
                    let command = tactical_state.command_buffer.trim().to_lowercase();
                    tactical_state.command_buffer.clear();

                    match command.as_str() {
                        "attack" | "a" => {
                            tactical_state.add_log("Attack command entered. Use 'a' hotkey for faster access.".to_string());
                            tactical_state.pending_action = Some(crate::ui::PendingAction::Attack { target_index: None });
                        }
                        "move" | "m" => {
                            tactical_state.add_log("Move command entered. Use 'm' hotkey for faster access.".to_string());
                            tactical_state.pending_action = Some(crate::ui::PendingAction::Move { destination: None });
                        }
                        "defend" | "d" => {
                            tactical_state.add_log("Defending...".to_string());
                            tactical_state.pending_action = Some(crate::ui::PendingAction::Defend);
                        }
                        "end" | "e" => {
                            tactical_state.add_log("Ending turn...".to_string());
                            tactical_state.pending_action = None;
                            self.advance_turn(&mut tactical_state);
                        }
                        "help" => {
                            tactical_state.add_log("Commands: attack, move, defend, end, quit".to_string());
                        }
                        "quit" | "q" => {
                            // Exit combat
                            if let Some(dungeon_state) = &tactical_state.return_to_dungeon {
                                self.state = UIState::DungeonExploration(dungeon_state.clone());
                            } else {
                                self.state = UIState::Playing;
                            }
                            return Ok(());
                        }
                        _ => {
                            tactical_state.add_log(format!("Unknown command: {}. Type 'help' for available commands.", command));
                        }
                    }

                    self.state = UIState::TacticalCombat(tactical_state);
                } else if tactical_state.pending_action.is_some() {
                    // Confirm pending action at cursor position
                    self.execute_tactical_action(&mut tactical_state)?;
                    self.state = UIState::TacticalCombat(tactical_state);
                }
            }

            // Backspace - delete character from command buffer
            KeyCode::Backspace => {
                tactical_state.command_buffer.pop();
                self.state = UIState::TacticalCombat(tactical_state);
            }

            // Escape - cancel pending action or exit combat
            KeyCode::Esc => {
                if tactical_state.pending_action.is_some() {
                    // Cancel pending action
                    tactical_state.pending_action = None;
                    tactical_state.add_log("Action cancelled.".to_string());
                    self.state = UIState::TacticalCombat(tactical_state);
                } else {
                    // Exit combat
                    if let Some(dungeon_state) = &tactical_state.return_to_dungeon {
                        self.state = UIState::DungeonExploration(dungeon_state.clone());
                    } else {
                        self.state = UIState::Playing;
                    }
                }
            }

            // Regular character input - add to command buffer
            KeyCode::Char(c) if !c.is_control() => {
                // Don't add hotkey letters if not in uppercase
                if !matches!(c, 'a'..='z') || tactical_state.command_buffer.len() > 0 {
                    tactical_state.command_buffer.push(c);
                    self.state = UIState::TacticalCombat(tactical_state);
                }
            }

            _ => {
                // Ignore other keys
                self.state = UIState::TacticalCombat(tactical_state);
            }
        }

        Ok(())
    }

    fn execute_tactical_action(&mut self, tactical_state: &mut crate::ui::TacticalCombatState) -> anyhow::Result<()> {
        use rand::Rng;

        let pending_action = tactical_state.pending_action.clone();

        match pending_action {
            Some(crate::ui::PendingAction::Attack { target_index: _ }) => {
                // Find target at cursor position
                let cursor_pos = tactical_state.cursor_pos;
                let target_idx = tactical_state.participants.iter().enumerate().find(|(idx, _)| {
                    tactical_state.battlefield.participant_positions.get(idx)
                        .map(|pos| pos.x == cursor_pos.0 && pos.y == cursor_pos.1)
                        .unwrap_or(false)
                }).map(|(idx, _)| idx);

                if let Some(target_idx) = target_idx {
                    let current_idx = tactical_state.initiative_order[tactical_state.current_turn_index].0;

                    // Extract all data we need before any mutable borrows
                    let attacker_name;
                    let defender_name;
                    let attack_value;
                    let defensive_value;
                    let has_weapon;
                    let damage_bonus;

                    {
                        let attacker = &tactical_state.participants[current_idx];
                        let defender = &tactical_state.participants[target_idx];

                        attacker_name = attacker.base_participant.name.clone();
                        defender_name = defender.base_participant.name.clone();
                        attack_value = attacker.base_participant.combat_stats.attack_value;
                        defensive_value = defender.base_participant.combat_stats.defensive_value;
                        has_weapon = attacker.base_participant.weapon.is_some();
                        damage_bonus = attacker.base_participant.combat_stats.damage_bonus;
                    }

                    // Roll attack
                    let mut rng = rand::thread_rng();
                    let attack_roll = rng.gen_range(1..=20);
                    let total_attack = attack_roll + attack_value as i32;

                    tactical_state.add_log(format!("{} attacks {}! (Roll: {}, Total: {})",
                        attacker_name, defender_name, attack_roll, total_attack));

                    if total_attack >= defensive_value as i32 {
                        // Hit! Calculate damage
                        // For now, use a simple d8 for weapons, d4 for unarmed
                        let base_damage = if has_weapon {
                            8 // d8 weapon damage
                        } else {
                            4 // d4 unarmed damage
                        };

                        let damage_roll = rng.gen_range(1..=base_damage);
                        let total_damage = (damage_roll as i32 + damage_bonus as i32).max(1) as u32;

                        // Apply damage
                        let defender = &mut tactical_state.participants[target_idx];
                        let old_hp = defender.base_participant.combat_stats.hit_points.current;
                        defender.base_participant.combat_stats.hit_points.current =
                            defender.base_participant.combat_stats.hit_points.current.saturating_sub(total_damage);
                        let new_hp = defender.base_participant.combat_stats.hit_points.current;

                        tactical_state.add_log(format!("HIT! {} damage dealt! ({} HP -> {} HP)",
                            total_damage, old_hp, new_hp));

                        if new_hp == 0 {
                            tactical_state.add_log(format!("{} has been defeated!", defender_name));
                        }
                    } else {
                        tactical_state.add_log(format!("MISS! (Needed {} to hit)", defensive_value));
                    }

                    // Clear pending action and advance turn
                    tactical_state.pending_action = None;
                    self.advance_turn(tactical_state);
                } else {
                    tactical_state.add_log("No valid target at cursor position!".to_string());
                }
            }

            Some(crate::ui::PendingAction::Move { destination: _ }) => {
                // Move to cursor position
                let cursor_pos = tactical_state.cursor_pos;
                let current_idx = tactical_state.initiative_order[tactical_state.current_turn_index].0;

                // Check if position is valid (not occupied, within movement range, etc.)
                let occupied = tactical_state.participants.iter().enumerate().any(|(idx, _)| {
                    if idx == current_idx {
                        return false;
                    }
                    tactical_state.battlefield.participant_positions.get(&idx)
                        .map(|pos| pos.x == cursor_pos.0 && pos.y == cursor_pos.1)
                        .unwrap_or(false)
                });

                if occupied {
                    tactical_state.add_log("Position is occupied!".to_string());
                } else {
                    // Move participant
                    let participant_name = tactical_state.participants[current_idx].base_participant.name.clone();
                    let old_pos = tactical_state.battlefield.participant_positions.get(&current_idx).cloned();
                    let new_pos = crate::forge::BattlefieldPosition::new(cursor_pos.0, cursor_pos.1);

                    tactical_state.battlefield.participant_positions.insert(current_idx, new_pos);
                    tactical_state.participants[current_idx].position = new_pos;

                    if let Some(old_p) = old_pos {
                        tactical_state.add_log(format!("{} moves from ({}, {}) to ({}, {})",
                            participant_name, old_p.x, old_p.y, cursor_pos.0, cursor_pos.1));
                    } else {
                        tactical_state.add_log(format!("{} moves to ({}, {})",
                            participant_name, cursor_pos.0, cursor_pos.1));
                    }

                    // Clear pending action and advance turn
                    tactical_state.pending_action = None;
                    self.advance_turn(tactical_state);
                }
            }

            Some(crate::ui::PendingAction::Defend) => {
                let current_idx = tactical_state.initiative_order[tactical_state.current_turn_index].0;
                let participant_name = tactical_state.participants[current_idx].base_participant.name.clone();
                tactical_state.add_log(format!("{} takes a defensive stance!", participant_name));

                // Clear pending action and advance turn
                tactical_state.pending_action = None;
                self.advance_turn(tactical_state);
            }

            Some(crate::ui::PendingAction::CastSpell { .. }) => {
                tactical_state.add_log("Spell casting not yet implemented!".to_string());
                tactical_state.pending_action = None;
            }

            Some(crate::ui::PendingAction::UseItem { .. }) => {
                tactical_state.add_log("Item usage not yet implemented!".to_string());
                tactical_state.pending_action = None;
            }

            None => {}
        }

        Ok(())
    }

    fn advance_turn(&mut self, tactical_state: &mut crate::ui::TacticalCombatState) {
        // Advance to next participant
        tactical_state.current_turn_index = (tactical_state.current_turn_index + 1) % tactical_state.initiative_order.len();

        // Check if we've completed a round
        if tactical_state.current_turn_index == 0 {
            tactical_state.round_number += 1;
            tactical_state.add_log(format!("=== ROUND {} ===", tactical_state.round_number));
        }

        // Log whose turn it is
        if let Some(participant) = tactical_state.get_current_participant() {
            tactical_state.add_log(format!("{}'s turn!", participant.base_participant.name));
        }

        // Process AI turns automatically until it's a player's turn
        self.process_ai_turns_until_player(tactical_state);
    }

    fn process_ai_turns_until_player(&mut self, tactical_state: &mut crate::ui::TacticalCombatState) {
        // Keep processing AI turns until we reach a player or all participants are processed
        let max_iterations = tactical_state.participants.len() * 2; // Safety limit
        let mut iterations = 0;

        while iterations < max_iterations {
            if let Some(current_participant) = tactical_state.get_current_participant() {
                // If it's a player's turn, stop and return control
                if current_participant.base_participant.is_player {
                    break;
                }

                // If participant is dead, skip their turn
                if current_participant.base_participant.combat_stats.hit_points.current == 0 {
                    tactical_state.add_log(format!("{} is defeated and cannot act.", current_participant.base_participant.name));

                    // Advance to next turn
                    tactical_state.current_turn_index = (tactical_state.current_turn_index + 1) % tactical_state.initiative_order.len();
                    if tactical_state.current_turn_index == 0 {
                        tactical_state.round_number += 1;
                        tactical_state.add_log(format!("=== ROUND {} ===", tactical_state.round_number));
                    }
                    if let Some(participant) = tactical_state.get_current_participant() {
                        tactical_state.add_log(format!("{}'s turn!", participant.base_participant.name));
                    }
                    iterations += 1;
                    continue;
                }

                // Execute AI turn
                self.execute_ai_turn(tactical_state);
            } else {
                break;
            }

            iterations += 1;
        }
    }

    fn execute_ai_turn(&mut self, tactical_state: &mut crate::ui::TacticalCombatState) {
        let current_idx = tactical_state.initiative_order[tactical_state.current_turn_index].0;

        // Find the player's position and index
        let player_info: Option<(usize, crate::forge::BattlefieldPosition)> = tactical_state.participants
            .iter()
            .enumerate()
            .find(|(_, p)| p.base_participant.is_player && p.base_participant.combat_stats.hit_points.current > 0)
            .and_then(|(idx, _)| {
                tactical_state.battlefield.participant_positions.get(&idx)
                    .map(|pos| (idx, pos.clone()))
            });

        if let Some((player_idx, player_pos)) = player_info {
            // Get AI position
            let ai_pos = tactical_state.battlefield.participant_positions.get(&current_idx).cloned();

            if let Some(ai_position) = ai_pos {
                // Calculate distance to player
                let dx = (player_pos.x - ai_position.x).abs();
                let dy = (player_pos.y - ai_position.y).abs();
                let distance = dx + dy; // Manhattan distance

                // If player is adjacent (distance 1), attack them
                if distance <= 1 {
                    // Execute attack on player
                    self.execute_ai_attack(tactical_state, current_idx, player_idx);
                } else {
                    // For now, AI just passes if not in range
                    // TODO: Add AI movement in next iteration
                    let ai_name = tactical_state.participants[current_idx].base_participant.name.clone();
                    tactical_state.add_log(format!("{} is too far to attack (distance: {}) and passes.", ai_name, distance));
                }
            }
        } else {
            // No valid player target
            let ai_name = tactical_state.participants[current_idx].base_participant.name.clone();
            tactical_state.add_log(format!("{} has no valid targets and passes.", ai_name));
        }

        // Advance to next turn (without triggering recursive AI processing)
        tactical_state.current_turn_index = (tactical_state.current_turn_index + 1) % tactical_state.initiative_order.len();
        if tactical_state.current_turn_index == 0 {
            tactical_state.round_number += 1;
            tactical_state.add_log(format!("=== ROUND {} ===", tactical_state.round_number));
        }
        if let Some(participant) = tactical_state.get_current_participant() {
            tactical_state.add_log(format!("{}'s turn!", participant.base_participant.name));
        }
    }

    fn execute_ai_attack(&mut self, tactical_state: &mut crate::ui::TacticalCombatState, attacker_idx: usize, target_idx: usize) {
        use rand::Rng;

        // Extract all data we need before any mutable borrows
        let attacker_name;
        let defender_name;
        let attack_value;
        let defensive_value;
        let has_weapon;
        let damage_bonus;

        {
            let attacker = &tactical_state.participants[attacker_idx];
            let defender = &tactical_state.participants[target_idx];

            attacker_name = attacker.base_participant.name.clone();
            defender_name = defender.base_participant.name.clone();
            attack_value = attacker.base_participant.combat_stats.attack_value;
            defensive_value = defender.base_participant.combat_stats.defensive_value;
            has_weapon = attacker.base_participant.weapon.is_some();
            damage_bonus = attacker.base_participant.combat_stats.damage_bonus;
        }

        // Roll attack
        let mut rng = rand::thread_rng();
        let attack_roll = rng.gen_range(1..=20);
        let total_attack = attack_roll + attack_value as i32;

        tactical_state.add_log(format!("{} attacks {}! (Roll: {}, Total: {})",
            attacker_name, defender_name, attack_roll, total_attack));

        if total_attack >= defensive_value as i32 {
            // Hit! Calculate damage
            let base_damage = if has_weapon { 8 } else { 4 }; // d8 or d4
            let damage_roll = rng.gen_range(1..=base_damage);
            let total_damage = (damage_roll as i32 + damage_bonus as i32).max(1) as u32;

            // Apply damage
            let defender = &mut tactical_state.participants[target_idx];
            let old_hp = defender.base_participant.combat_stats.hit_points.current;
            defender.base_participant.combat_stats.hit_points.current =
                defender.base_participant.combat_stats.hit_points.current.saturating_sub(total_damage);
            let new_hp = defender.base_participant.combat_stats.hit_points.current;

            tactical_state.add_log(format!("HIT! {} damage dealt! ({} HP -> {} HP)",
                total_damage, old_hp, new_hp));

            if new_hp == 0 {
                tactical_state.add_log(format!("{} has been defeated!", defender_name));
            }
        } else {
            tactical_state.add_log(format!("MISS! (Needed {} to hit)", defensive_value));
        }
    }

    fn get_compatible_items_static(character: &crate::forge::ForgeCharacter, slot: &crate::ui::EquipmentSlot) -> Vec<crate::forge::InventoryItem> {
        character.inventory.items.iter()
            .filter(|item| {
                match (slot, &item.item_type) {
                    (crate::ui::EquipmentSlot::Weapon, crate::forge::ItemType::Weapon(_)) => true,
                    (crate::ui::EquipmentSlot::Armor, crate::forge::ItemType::Armor(armor)) => !armor.name.contains("Shield"),
                    (crate::ui::EquipmentSlot::Shield, crate::forge::ItemType::Armor(armor)) => armor.name.contains("Shield"),
                    (crate::ui::EquipmentSlot::Accessory1 | crate::ui::EquipmentSlot::Accessory2, crate::forge::ItemType::Accessory(_)) => true,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

}