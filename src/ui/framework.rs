use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// Z-index ordering for UI layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZIndex {
    Background = 0,
    MainContent = 10,
    Sidebar = 20,
    Modal = 30,
    Tooltip = 40,
    SystemOverlay = 50,
}

/// Modal size presets
#[derive(Debug, Clone, Copy)]
pub enum ModalSize {
    Small,      // 40% x 50%
    Medium,     // 60% x 70%
    Large,      // 80% x 85%
    FullScreen, // 95% x 90%
    Custom(u16, u16), // width%, height%
}

impl ModalSize {
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            ModalSize::Small => (40, 50),
            ModalSize::Medium => (60, 70),
            ModalSize::Large => (80, 85),
            ModalSize::FullScreen => (95, 90),
            ModalSize::Custom(w, h) => (*w, *h),
        }
    }
}

/// Theme system for consistent styling
#[derive(Debug, Clone)]
pub struct UITheme {
    pub background: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub border_primary: Color,
    pub border_secondary: Color,
}

impl Default for UITheme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            primary: Color::Blue,
            secondary: Color::Gray,
            accent: Color::Yellow,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text_primary: Color::White,
            text_secondary: Color::Gray,
            border_primary: Color::White,
            border_secondary: Color::Gray,
        }
    }
}

/// Base component trait for all UI elements
pub trait UIComponent {
    fn render(&self, f: &mut Frame, area: Rect, theme: &UITheme);
    fn get_z_index(&self) -> ZIndex;
    fn is_visible(&self) -> bool { true }
    fn get_id(&self) -> Option<&str> { None }
}

/// Modal component with proper layering and background blocking
#[derive(Debug, Clone)]
pub struct Modal {
    pub id: String,
    pub title: String,
    pub size: ModalSize,
    pub content: ModalContent,
    pub closable: bool,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum ModalContent {
    Text(Vec<Line<'static>>),
    List(Vec<ListItem<'static>>),
    Custom(String), // For custom rendering
}

impl Modal {
    pub fn new(id: String, title: String, size: ModalSize) -> Self {
        Self {
            id,
            title,
            size,
            content: ModalContent::Text(vec![]),
            closable: true,
            visible: true,
        }
    }

    pub fn with_content(mut self, content: ModalContent) -> Self {
        self.content = content;
        self
    }

    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Create centered rect for modal
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl UIComponent for Modal {
    fn render(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        if !self.visible {
            return;
        }

        let (width, height) = self.size.dimensions();
        let modal_area = Self::centered_rect(width, height, area);

        // First, render a solid background to block underlying content
        let background_block = Block::default()
            .style(Style::default().bg(theme.background));
        f.render_widget(background_block, modal_area);

        // Then render the modal border and title
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone())
            .border_style(Style::default().fg(theme.border_primary))
            .style(Style::default().bg(theme.background));
        f.render_widget(modal_block, modal_area);

        // Content area (inside the border)
        let content_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        // Render content based on type
        match &self.content {
            ModalContent::Text(lines) => {
                let paragraph = Paragraph::new(lines.clone())
                    .style(Style::default().fg(theme.text_primary).bg(theme.background))
                    .alignment(Alignment::Left);
                f.render_widget(paragraph, content_area);
            }
            ModalContent::List(items) => {
                let list = List::new(items.clone())
                    .style(Style::default().fg(theme.text_primary).bg(theme.background));
                f.render_widget(list, content_area);
            }
            ModalContent::Custom(_) => {
                // Custom content handled by specialized renderers
            }
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

/// Panel component for organized content areas
#[derive(Debug, Clone)]
pub struct Panel {
    pub id: String,
    pub title: Option<String>,
    pub content: Vec<Line<'static>>,
    pub border_style: Option<Style>,
    pub content_style: Option<Style>,
    pub visible: bool,
}

impl Panel {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: None,
            content: vec![],
            border_style: None,
            content_style: None,
            visible: true,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_content(mut self, content: Vec<Line<'static>>) -> Self {
        self.content = content;
        self
    }

    pub fn with_border_style(mut self, style: Style) -> Self {
        self.border_style = Some(style);
        self
    }

    pub fn with_content_style(mut self, style: Style) -> Self {
        self.content_style = Some(style);
        self
    }
}

impl UIComponent for Panel {
    fn render(&self, f: &mut Frame, area: Rect, theme: &UITheme) {
        if !self.visible {
            return;
        }

        let mut block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.background));

        if let Some(title) = &self.title {
            block = block.title(title.clone());
        }

        if let Some(border_style) = self.border_style {
            block = block.border_style(border_style);
        } else {
            block = block.border_style(Style::default().fg(theme.border_primary));
        }

        let paragraph = Paragraph::new(self.content.clone())
            .block(block)
            .style(
                self.content_style
                    .unwrap_or(Style::default().fg(theme.text_primary).bg(theme.background))
            );

        f.render_widget(paragraph, area);
    }

    fn get_z_index(&self) -> ZIndex {
        ZIndex::MainContent
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn get_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// UI Manager for handling component lifecycle and rendering
pub struct UIManager {
    components: HashMap<String, Box<dyn UIComponent>>,
    render_order: Vec<String>,
    theme: UITheme,
    modal_stack: Vec<String>,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            render_order: Vec::new(),
            theme: UITheme::default(),
            modal_stack: Vec::new(),
        }
    }

    pub fn with_theme(mut self, theme: UITheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn add_component(&mut self, component: Box<dyn UIComponent>) {
        if let Some(id) = component.get_id() {
            let id = id.to_string();
            let z_index = component.get_z_index();
            
            self.components.insert(id.clone(), component);
            
            // Insert in correct z-index order
            let insert_pos = self.render_order
                .iter()
                .position(|existing_id| {
                    self.components.get(existing_id)
                        .map(|c| c.get_z_index() > z_index)
                        .unwrap_or(true)
                })
                .unwrap_or(self.render_order.len());
            
            self.render_order.insert(insert_pos, id);
        }
    }

    pub fn remove_component(&mut self, id: &str) {
        self.components.remove(id);
        self.render_order.retain(|x| x != id);
        self.modal_stack.retain(|x| x != id);
    }

    pub fn show_modal(&mut self, id: &str) {
        if self.components.contains_key(id) && !self.modal_stack.contains(&id.to_string()) {
            self.modal_stack.push(id.to_string());
        }
    }

    pub fn close_modal(&mut self, id: &str) {
        self.modal_stack.retain(|x| x != id);
    }

    pub fn close_top_modal(&mut self) -> Option<String> {
        self.modal_stack.pop()
    }

    pub fn get_active_modal(&self) -> Option<&str> {
        self.modal_stack.last().map(|s| s.as_str())
    }

    pub fn render_all(&self, f: &mut Frame, area: Rect) {
        // Render components in z-index order
        for id in &self.render_order {
            if let Some(component) = self.components.get(id) {
                if component.is_visible() {
                    component.render(f, area, &self.theme);
                }
            }
        }

        // Render active modals on top
        for modal_id in &self.modal_stack {
            if let Some(component) = self.components.get(modal_id) {
                if component.is_visible() {
                    component.render(f, area, &self.theme);
                }
            }
        }
    }

    pub fn update_theme(&mut self, theme: UITheme) {
        self.theme = theme;
    }

    pub fn get_theme(&self) -> &UITheme {
        &self.theme
    }

    /// Validate UI state for consistency
    pub fn validate_state(&self) -> Result<(), String> {
        // Check for orphaned render order entries
        for id in &self.render_order {
            if !self.components.contains_key(id) {
                return Err(format!("Orphaned render order entry: {}", id));
            }
        }

        // Check for orphaned modal stack entries
        for id in &self.modal_stack {
            if !self.components.contains_key(id) {
                return Err(format!("Orphaned modal stack entry: {}", id));
            }
        }

        // Check for duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for id in self.components.keys() {
            if !seen_ids.insert(id) {
                return Err(format!("Duplicate component ID: {}", id));
            }
        }

        Ok(())
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for common UI patterns
pub mod helpers {
    use super::*;

    /// Create a spell selection modal
    pub fn create_spell_selection_modal(
        spells: Vec<(String, String)>, // (name, description) pairs
        selected_index: usize,
    ) -> Modal {
        let spell_items: Vec<ListItem> = spells
            .iter()
            .enumerate()
            .map(|(i, (name, desc))| {
                let style = if i == selected_index {
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(vec![
                    Line::from(vec![Span::styled(name.clone(), style)]),
                    Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled(desc.clone(), Style::default().fg(Color::Gray)),
                    ]),
                ])
            })
            .collect();

        Modal::new(
            "spell_selection".to_string(),
            "🔮 Forge Spell Casting System 🔮".to_string(),
            ModalSize::Large,
        )
        .with_content(ModalContent::List(spell_items))
    }

    /// Create an action selection modal
    pub fn create_action_selection_modal(
        participant_name: String,
        hp_current: u32,
        hp_max: u32,
        av: i32,
        dv: i32,
    ) -> Modal {
        let content = vec![
            Line::from(vec![
                Span::styled("Current: ".to_string(), Style::default().fg(Color::White)),
                Span::styled(participant_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]),
            Line::from(format!("HP: {}/{}", hp_current, hp_max)),
            Line::from(format!("AV: {} | DV: {}", av, dv)),
            Line::from(""),
            Line::from("Choose your action for this combat minute:"),
        ];

        Modal::new(
            "action_selection".to_string(),
            "Declare Action for Combat Minute".to_string(),
            ModalSize::Medium,
        )
        .with_content(ModalContent::Text(content))
    }

    /// Create an information panel
    pub fn create_info_panel(id: String, title: String, info_lines: Vec<Line<'static>>) -> Panel {
        Panel::new(id)
            .with_title(title)
            .with_content(info_lines)
            .with_border_style(Style::default().fg(Color::Blue))
    }
}