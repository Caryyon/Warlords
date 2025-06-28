use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use super::framework::{UIComponent, UITheme, ZIndex};

/// Spell selection component with proper modal behavior
#[derive(Debug, Clone)]
pub struct SpellSelectionComponent {
    pub id: String,
    pub spells: Vec<SpellInfo>,
    pub selected_index: usize,
    pub visible: bool,
    pub enhanced_mode: bool,
    pub current_enhancement: SpellEnhancement,
}

#[derive(Debug, Clone)]
pub struct SpellInfo {
    pub name: String,
    pub school: String,
    pub level: u8,
    pub cost: u8,
    pub description: String,
    pub success_rate: u8,
    pub backfire_risk: u8,
    pub range: u8,
    pub enhancement_cost: u8,
    pub max_pumps: u8,
    pub component_break_chance: u8,
}

#[derive(Debug, Clone, Default)]
pub struct SpellEnhancement {
    pub pumps: u8,
    pub enhanced_range: bool,
    pub enhanced_duration: bool,
    pub enhanced_damage: bool,
    pub enhanced_save_modifier: bool,
    pub enhanced_success_chance: bool,
    pub total_cost: u8,
}

impl SpellSelectionComponent {
    pub fn new(id: String, spells: Vec<SpellInfo>) -> Self {
        Self {
            id,
            spells,
            selected_index: 0,
            visible: false,
            enhanced_mode: false,
            current_enhancement: SpellEnhancement::default(),
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.enhanced_mode = false;
        self.current_enhancement = SpellEnhancement::default();
    }

    pub fn select_next(&mut self) {
        if !self.spells.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.spells.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.spells.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.spells.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected_spell(&self) -> Option<&SpellInfo> {
        self.spells.get(self.selected_index)
    }

    pub fn enter_enhancement_mode(&mut self) {
        self.enhanced_mode = true;
    }

    pub fn exit_enhancement_mode(&mut self) {
        self.enhanced_mode = false;
        self.current_enhancement = SpellEnhancement::default();
    }

    fn render_spell_list(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        let spell_items: Vec<ListItem> = self.spells
            .iter()
            .enumerate()
            .map(|(i, spell)| {
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Black).bg(theme.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_primary).bg(theme.background)
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!("{} ({} SP)", spell.name, spell.cost), style),
                    ]),
                    Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled(spell.school.clone(), Style::default().fg(theme.text_secondary).bg(theme.background)),
                    ]),
                ])
            })
            .collect();

        let spell_list = List::new(spell_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("📜 Available Spells")
                .border_style(Style::default().fg(theme.border_primary))
                .style(Style::default().bg(theme.background)));
        
        f.render_widget(spell_list, area);
    }

    fn render_spell_details(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        if let Some(spell) = self.get_selected_spell() {
            let details = vec![
                Line::from(vec![
                    Span::styled("Level: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(spell.level.to_string(), Style::default().fg(theme.text_primary)),
                ]),
                Line::from(vec![
                    Span::styled("Base Cost: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} SP", spell.cost), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Success Rate: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{}%", spell.success_rate), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Backfire Risk: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{}%", spell.backfire_risk), Style::default().fg(Color::Red)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description:", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(spell.description.clone(), Style::default().fg(Color::LightBlue)),
                ]),
            ];

            let details_widget = Paragraph::new(details)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("✨ Spell Details")
                    .border_style(Style::default().fg(Color::Blue))
                    .style(Style::default().bg(theme.background)));
            f.render_widget(details_widget, area);
        }
    }

    fn render_enhancement_interface(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        if !self.enhanced_mode {
            return;
        }

        if let Some(spell) = self.get_selected_spell() {
            let enhancement_options = vec![
                Line::from("🔮 Enhancement Options:"),
                Line::from(""),
                Line::from(format!("📏 Range: Extend to {} tiles", spell.range + 5)),
                Line::from("⏱️  Duration: +3 rounds"),
                Line::from("💥 Damage: +2 damage/healing"),
                Line::from("🛡️  Save Modifier: -1 to enemy saves"),
                Line::from("🎯 Success: +15% casting chance"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Cost per enhancement: ", Style::default().fg(theme.text_primary)),
                    Span::styled(format!("+{} SP", spell.enhancement_cost), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::styled("Current total: ", Style::default().fg(theme.text_primary)),
                    Span::styled(format!("{} SP", self.current_enhancement.total_cost), Style::default().fg(Color::Green)),
                ]),
            ];

            let enhancement_widget = Paragraph::new(enhancement_options)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("⚡ Spell Enhancement")
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(theme.background)));
            f.render_widget(enhancement_widget, area);
        }
    }
}

impl UIComponent for SpellSelectionComponent {
    fn render(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        if !self.visible {
            return;
        }

        // Create modal area
        let modal_area = centered_rect(90, 80, area);

        // Render background
        let background_block = Block::default()
            .style(Style::default().bg(theme.background));
        f.render_widget(background_block, modal_area);

        // Render border
        let border_block = Block::default()
            .borders(Borders::ALL)
            .title("🔮 Forge Spell Casting System 🔮")
            .border_style(Style::default().fg(Color::Magenta))
            .style(Style::default().bg(theme.background));
        f.render_widget(border_block, modal_area);

        // Content area
        let content_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        if self.enhanced_mode {
            // Enhancement mode layout
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(40),  // Spell list
                    ratatui::layout::Constraint::Percentage(30),  // Details
                    ratatui::layout::Constraint::Percentage(30),  // Enhancement
                ])
                .split(content_area);

            self.render_spell_list(f, chunks[0], theme);
            self.render_spell_details(f, chunks[1], theme);
            self.render_enhancement_interface(f, chunks[2], theme);
        } else {
            // Normal mode layout
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Percentage(50),  // Spell list
                    ratatui::layout::Constraint::Percentage(50),  // Details
                ])
                .split(content_area);

            self.render_spell_list(f, chunks[0], theme);
            self.render_spell_details(f, chunks[1], theme);
        }
    }

    fn get_z_index(&self) -> ZIndex {
        ZIndex::Modal
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn get_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// Action selection component for combat
#[derive(Debug, Clone)]
pub struct ActionSelectionComponent {
    pub id: String,
    pub participant_name: String,
    pub hp_current: u32,
    pub hp_max: u32,
    pub av: i32,
    pub dv: i32,
    pub actions: Vec<ActionInfo>,
    pub selected_index: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct ActionInfo {
    pub key: String,
    pub name: String,
    pub description: String,
    pub available: bool,
}

impl ActionSelectionComponent {
    pub fn new(
        id: String,
        participant_name: String,
        hp_current: u32,
        hp_max: u32,
        av: i32,
        dv: i32,
    ) -> Self {
        let actions = vec![
            ActionInfo {
                key: "1".to_string(),
                name: "Melee Attack".to_string(),
                description: "Attack an adjacent enemy with weapon".to_string(),
                available: true,
            },
            ActionInfo {
                key: "2".to_string(),
                name: "Missile Attack".to_string(),
                description: "Shoot at distant enemy".to_string(),
                available: true,
            },
            ActionInfo {
                key: "3".to_string(),
                name: "Cast Spell".to_string(),
                description: "Use magical abilities".to_string(),
                available: true,
            },
            ActionInfo {
                key: "4".to_string(),
                name: "Defend".to_string(),
                description: "Designate prime opponent, gain DV bonus".to_string(),
                available: true,
            },
            ActionInfo {
                key: "5".to_string(),
                name: "Use Item".to_string(),
                description: "Consume item (potion, scroll, etc.)".to_string(),
                available: true,
            },
            ActionInfo {
                key: "6".to_string(),
                name: "Wait".to_string(),
                description: "Take no action this minute".to_string(),
                available: true,
            },
        ];

        Self {
            id,
            participant_name,
            hp_current,
            hp_max,
            av,
            dv,
            actions,
            selected_index: 0,
            visible: false,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn get_selected_action(&self) -> Option<&ActionInfo> {
        self.actions.get(self.selected_index)
    }
}

impl UIComponent for ActionSelectionComponent {
    fn render(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        if !self.visible {
            return;
        }

        // Create modal area
        let modal_area = centered_rect(60, 70, area);

        // Render background
        let background_block = Block::default()
            .style(Style::default().bg(theme.background));
        f.render_widget(background_block, modal_area);

        // Render border
        let border_block = Block::default()
            .borders(Borders::ALL)
            .title("Declare Action for Combat Minute")
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(theme.background));
        f.render_widget(border_block, modal_area);

        // Content area
        let content_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(6),   // Participant info
                ratatui::layout::Constraint::Min(0),      // Actions
                ratatui::layout::Constraint::Length(4),   // Instructions
            ])
            .split(content_area);

        // Participant info
        let participant_info = vec![
            Line::from(vec![
                Span::styled("Current: ", Style::default().fg(theme.text_primary)),
                Span::styled(&self.participant_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]),
            Line::from(format!("HP: {}/{}", self.hp_current, self.hp_max)),
            Line::from(format!("AV: {} | DV: {}", self.av, self.dv)),
            Line::from(""),
            Line::from("Choose your action for this combat minute:"),
        ];

        let info_widget = Paragraph::new(participant_info)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Current Participant")
                .style(Style::default().bg(theme.background)));
        f.render_widget(info_widget, chunks[0]);

        // Actions list
        let action_items: Vec<ListItem> = self.actions.iter().map(|action| {
            let style = if action.available {
                Style::default().fg(theme.text_primary)
            } else {
                Style::default().fg(theme.text_secondary)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{}: ", action.key), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(action.name.clone(), style.add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("    ", style),
                    Span::styled(action.description.clone(), Style::default().fg(Color::Gray)),
                ]),
            ])
        }).collect();

        let actions_list = List::new(action_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Available Actions")
                .border_style(Style::default().fg(Color::Green))
                .style(Style::default().bg(theme.background)));
        f.render_widget(actions_list, chunks[1]);

        // Instructions
        let instructions = vec![
            Line::from("Press 1-6 to select action | TAB: Skip to next participant"),
            Line::from("ESC: Exit Forge combat | F: Toggle full Forge mode"),
        ];

        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(theme.text_secondary).bg(theme.background))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .style(Style::default().bg(theme.background)));
        f.render_widget(instructions_widget, chunks[2]);
    }

    fn get_z_index(&self) -> ZIndex {
        ZIndex::Modal
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn get_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// Helper function for creating centered rectangles
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}