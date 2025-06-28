use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
};

/// Layout manager for consistent screen layouts across different game states
#[derive(Debug, Clone)]
pub struct LayoutManager {
    pub screen_layout: ScreenLayout,
    pub tactical_layout: TacticalLayout,
    pub modal_constraints: ModalConstraints,
}

/// Main screen layout configurations
#[derive(Debug, Clone)]
pub struct ScreenLayout {
    pub header_height: u16,
    pub footer_height: u16,
    pub sidebar_width: u16,
    pub main_content_padding: u16,
}

/// Tactical combat specific layout
#[derive(Debug, Clone)]
pub struct TacticalLayout {
    pub battlefield_width_percent: u16,
    pub info_panel_width_percent: u16,
    pub action_panel_height: u16,
    pub log_panel_height: u16,
}

/// Modal dialog constraints
#[derive(Debug, Clone)]
pub struct ModalConstraints {
    pub min_width: u16,
    pub min_height: u16,
    pub max_width_percent: u16,
    pub max_height_percent: u16,
    pub padding: u16,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self {
            screen_layout: ScreenLayout {
                header_height: 3,
                footer_height: 3,
                sidebar_width: 25,
                main_content_padding: 1,
            },
            tactical_layout: TacticalLayout {
                battlefield_width_percent: 45,
                info_panel_width_percent: 55,
                action_panel_height: 8,
                log_panel_height: 12,
            },
            modal_constraints: ModalConstraints {
                min_width: 40,
                min_height: 20,
                max_width_percent: 95,
                max_height_percent: 90,
                padding: 2,
            },
        }
    }
}

impl LayoutManager {
    /// Create a new layout manager with custom configurations
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate main game screen layout
    pub fn main_screen_layout(&self, area: Rect) -> MainScreenAreas {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.screen_layout.header_height),
                Constraint::Min(0),
                Constraint::Length(self.screen_layout.footer_height),
            ])
            .split(area);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100 - self.screen_layout.sidebar_width),
                Constraint::Percentage(self.screen_layout.sidebar_width),
            ])
            .split(main_chunks[1]);

        MainScreenAreas {
            header: main_chunks[0],
            main_content: content_chunks[0],
            sidebar: content_chunks[1],
            footer: main_chunks[2],
        }
    }

    /// Generate tactical combat layout
    pub fn tactical_combat_layout(&self, area: Rect) -> TacticalCombatAreas {
        // Main horizontal split: battlefield vs info panels
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.tactical_layout.battlefield_width_percent),
                Constraint::Percentage(self.tactical_layout.info_panel_width_percent),
            ])
            .split(area);

        // Right side vertical split: info panels
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Current participant info
                Constraint::Min(0),    // Actions/spells area
                Constraint::Length(self.tactical_layout.log_panel_height), // Combat log
                Constraint::Length(4), // Controls
            ])
            .split(main_chunks[1]);

        // Battlefield area (left side)
        let battlefield_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Actual battlefield
            ])
            .split(main_chunks[0]);

        TacticalCombatAreas {
            title: battlefield_chunks[0],
            battlefield: battlefield_chunks[1],
            participant_info: right_chunks[0],
            actions_area: right_chunks[1],
            combat_log: right_chunks[2],
            controls: right_chunks[3],
        }
    }

    /// Generate world exploration layout
    pub fn world_exploration_layout(&self, area: Rect) -> WorldExplorationAreas {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(75), // World view
                Constraint::Percentage(25), // Info panels
            ])
            .split(area);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Character info
                Constraint::Min(0),     // Messages/log
                Constraint::Length(4),  // Controls
            ])
            .split(main_chunks[1]);

        WorldExplorationAreas {
            world_view: main_chunks[0],
            character_info: right_chunks[0],
            messages: right_chunks[1],
            controls: right_chunks[2],
        }
    }

    /// Generate character creation layout
    pub fn character_creation_layout(&self, area: Rect) -> CharacterCreationAreas {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Main creation area
                Constraint::Percentage(40), // Info/preview area
            ])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Creation steps
                Constraint::Length(4),  // Controls
            ])
            .split(main_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Character preview
                Constraint::Percentage(50), // Race/class info
            ])
            .split(main_chunks[1]);

        CharacterCreationAreas {
            title: left_chunks[0],
            creation_area: left_chunks[1],
            controls: left_chunks[2],
            character_preview: right_chunks[0],
            info_panel: right_chunks[1],
        }
    }

    /// Create a centered modal area
    pub fn centered_modal(&self, width_percent: u16, height_percent: u16, area: Rect) -> Rect {
        let width = width_percent.clamp(
            self.modal_constraints.min_width,
            self.modal_constraints.max_width_percent,
        );
        let height = height_percent.clamp(
            self.modal_constraints.min_height,
            self.modal_constraints.max_height_percent,
        );

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - height) / 2),
                Constraint::Percentage(height),
                Constraint::Percentage((100 - height) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - width) / 2),
                Constraint::Percentage(width),
                Constraint::Percentage((100 - width) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Create a side panel (anchored to edge)
    pub fn side_panel(&self, side: PanelSide, width_percent: u16, area: Rect) -> Rect {
        match side {
            PanelSide::Left => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(width_percent),
                        Constraint::Percentage(100 - width_percent),
                    ])
                    .split(area);
                chunks[0]
            }
            PanelSide::Right => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(100 - width_percent),
                        Constraint::Percentage(width_percent),
                    ])
                    .split(area);
                chunks[1]
            }
            PanelSide::Top => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(width_percent),
                        Constraint::Percentage(100 - width_percent),
                    ])
                    .split(area);
                chunks[0]
            }
            PanelSide::Bottom => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(100 - width_percent),
                        Constraint::Percentage(width_percent),
                    ])
                    .split(area);
                chunks[1]
            }
        }
    }

    /// Validate layout constraints
    pub fn validate_constraints(&self) -> Result<(), String> {
        // Check tactical layout
        if self.tactical_layout.battlefield_width_percent + self.tactical_layout.info_panel_width_percent != 100 {
            return Err("Tactical layout width percentages must sum to 100".to_string());
        }

        // Check modal constraints
        if self.modal_constraints.min_width > self.modal_constraints.max_width_percent {
            return Err("Modal min_width cannot exceed max_width_percent".to_string());
        }

        if self.modal_constraints.min_height > self.modal_constraints.max_height_percent {
            return Err("Modal min_height cannot exceed max_height_percent".to_string());
        }

        Ok(())
    }
}

/// Layout area definitions
#[derive(Debug, Clone)]
pub struct MainScreenAreas {
    pub header: Rect,
    pub main_content: Rect,
    pub sidebar: Rect,
    pub footer: Rect,
}

#[derive(Debug, Clone)]
pub struct TacticalCombatAreas {
    pub title: Rect,
    pub battlefield: Rect,
    pub participant_info: Rect,
    pub actions_area: Rect,
    pub combat_log: Rect,
    pub controls: Rect,
}

#[derive(Debug, Clone)]
pub struct WorldExplorationAreas {
    pub world_view: Rect,
    pub character_info: Rect,
    pub messages: Rect,
    pub controls: Rect,
}

#[derive(Debug, Clone)]
pub struct CharacterCreationAreas {
    pub title: Rect,
    pub creation_area: Rect,
    pub controls: Rect,
    pub character_preview: Rect,
    pub info_panel: Rect,
}

#[derive(Debug, Clone, Copy)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Responsive layout utilities
pub struct ResponsiveLayout;

impl ResponsiveLayout {
    /// Adjust layout based on terminal size
    pub fn adjust_for_size(mut layout: LayoutManager, width: u16, height: u16) -> LayoutManager {
        // Small terminal adjustments
        if width < 80 || height < 24 {
            layout.screen_layout.sidebar_width = 20;
            layout.tactical_layout.battlefield_width_percent = 50;
            layout.tactical_layout.info_panel_width_percent = 50;
        }

        // Very large terminal optimizations
        if width > 120 {
            layout.tactical_layout.battlefield_width_percent = 40;
            layout.tactical_layout.info_panel_width_percent = 60;
        }

        layout
    }

    /// Calculate minimum required terminal size
    pub fn minimum_size() -> (u16, u16) {
        (60, 20) // 60 columns, 20 rows minimum
    }

    /// Check if current terminal size is adequate
    pub fn is_size_adequate(width: u16, height: u16) -> bool {
        let (min_width, min_height) = Self::minimum_size();
        width >= min_width && height >= min_height
    }
}

/// Color scheme for consistent UI theming
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub text: Color,
    pub border: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Gray,
            accent: Color::Yellow,
            background: Color::Black,
            text: Color::White,
            border: Color::White,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
        }
    }
}

impl ColorScheme {
    /// Create a dark theme
    pub fn dark() -> Self {
        Self::default()
    }

    /// Create a light theme (if terminal supports it)
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::DarkGray,
            accent: Color::Magenta,
            background: Color::White,
            text: Color::Black,
            border: Color::Black,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
        }
    }

    /// Get style for different UI elements
    pub fn get_style(&self, element: UIElement) -> Style {
        match element {
            UIElement::Title => Style::default()
                .fg(self.accent)
                .bg(self.background)
                .add_modifier(ratatui::style::Modifier::BOLD),
            UIElement::Border => Style::default()
                .fg(self.border)
                .bg(self.background),
            UIElement::Text => Style::default()
                .fg(self.text)
                .bg(self.background),
            UIElement::Selected => Style::default()
                .fg(Color::Black)
                .bg(self.accent)
                .add_modifier(ratatui::style::Modifier::BOLD),
            UIElement::Success => Style::default()
                .fg(self.success)
                .bg(self.background),
            UIElement::Warning => Style::default()
                .fg(self.warning)
                .bg(self.background),
            UIElement::Error => Style::default()
                .fg(self.error)
                .bg(self.background),
            UIElement::Info => Style::default()
                .fg(self.info)
                .bg(self.background),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UIElement {
    Title,
    Border,
    Text,
    Selected,
    Success,
    Warning,
    Error,
    Info,
}