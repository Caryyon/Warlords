use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::Rect;

/// Mouse event handling for TUI components
#[derive(Debug, Clone)]
pub struct MouseHandler {
    pub last_click: Option<MouseClick>,
    pub hover_position: Option<(u16, u16)>,
    pub drag_state: Option<DragState>,
}

#[derive(Debug, Clone)]
pub struct MouseClick {
    pub column: u16,
    pub row: u16,
    pub button: MouseButton,
    pub modifiers: crossterm::event::KeyModifiers,
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub start: (u16, u16),
    pub current: (u16, u16),
    pub button: MouseButton,
}

/// Result of mouse interaction with UI components
#[derive(Debug, Clone)]
pub enum MouseInteraction {
    None,
    ButtonClicked(String),  // Button ID that was clicked
    ListItemSelected(usize), // Index of list item selected
    AreaClicked(String, u16, u16), // Area ID and local coordinates
    Scroll(ScrollDirection, u16, u16), // Direction and position
    Drag(String, (u16, u16), (u16, u16)), // Component ID, start, end
}

#[derive(Debug, Clone)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Clickable UI element definition
#[derive(Debug, Clone)]
pub struct ClickableArea {
    pub id: String,
    pub area: Rect,
    pub element_type: ClickableElementType,
}

#[derive(Debug, Clone)]
pub enum ClickableElementType {
    Button,
    ListItem(usize),
    Scroll,
    Battlefield,
    MenuItem,
    TabHeader,
    CloseButton,
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            last_click: None,
            hover_position: None,
            drag_state: None,
        }
    }

    /// Process a mouse event and return any interactions
    pub fn handle_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        clickable_areas: &[ClickableArea],
    ) -> MouseInteraction {
        match mouse_event.kind {
            MouseEventKind::Down(button) => {
                self.last_click = Some(MouseClick {
                    column: mouse_event.column,
                    row: mouse_event.row,
                    button,
                    modifiers: mouse_event.modifiers,
                });

                // Check if click is within any clickable area
                self.find_clicked_element(mouse_event.column, mouse_event.row, clickable_areas)
            }
            MouseEventKind::Up(_button) => {
                // End any drag operation
                if let Some(drag) = &self.drag_state {
                    let interaction = MouseInteraction::Drag(
                        "drag_end".to_string(),
                        drag.start,
                        (mouse_event.column, mouse_event.row),
                    );
                    self.drag_state = None;
                    return interaction;
                }
                MouseInteraction::None
            }
            MouseEventKind::Drag(button) => {
                if let Some(ref mut drag) = self.drag_state {
                    drag.current = (mouse_event.column, mouse_event.row);
                } else {
                    // Start new drag if we have a recent click
                    if let Some(ref click) = self.last_click {
                        self.drag_state = Some(DragState {
                            start: (click.column, click.row),
                            current: (mouse_event.column, mouse_event.row),
                            button,
                        });
                    }
                }
                MouseInteraction::None
            }
            MouseEventKind::Moved => {
                self.hover_position = Some((mouse_event.column, mouse_event.row));
                MouseInteraction::None
            }
            MouseEventKind::ScrollDown => MouseInteraction::Scroll(
                ScrollDirection::Down,
                mouse_event.column,
                mouse_event.row,
            ),
            MouseEventKind::ScrollUp => MouseInteraction::Scroll(
                ScrollDirection::Up,
                mouse_event.column,
                mouse_event.row,
            ),
            MouseEventKind::ScrollLeft => MouseInteraction::Scroll(
                ScrollDirection::Left,
                mouse_event.column,
                mouse_event.row,
            ),
            MouseEventKind::ScrollRight => MouseInteraction::Scroll(
                ScrollDirection::Right,
                mouse_event.column,
                mouse_event.row,
            ),
        }
    }

    /// Find which clickable element was clicked
    fn find_clicked_element(
        &self,
        column: u16,
        row: u16,
        clickable_areas: &[ClickableArea],
    ) -> MouseInteraction {
        for area in clickable_areas {
            if self.is_point_in_rect(column, row, &area.area) {
                return match &area.element_type {
                    ClickableElementType::Button => {
                        MouseInteraction::ButtonClicked(area.id.clone())
                    }
                    ClickableElementType::ListItem(index) => {
                        MouseInteraction::ListItemSelected(*index)
                    }
                    ClickableElementType::Battlefield => {
                        // Convert to local battlefield coordinates
                        let local_x = column.saturating_sub(area.area.x);
                        let local_y = row.saturating_sub(area.area.y);
                        MouseInteraction::AreaClicked(area.id.clone(), local_x, local_y)
                    }
                    ClickableElementType::MenuItem => {
                        MouseInteraction::ButtonClicked(area.id.clone())
                    }
                    ClickableElementType::TabHeader => {
                        MouseInteraction::ButtonClicked(area.id.clone())
                    }
                    ClickableElementType::CloseButton => {
                        MouseInteraction::ButtonClicked("close".to_string())
                    }
                    ClickableElementType::Scroll => {
                        MouseInteraction::AreaClicked(area.id.clone(), column, row)
                    }
                };
            }
        }
        MouseInteraction::None
    }

    /// Check if a point is within a rectangle
    fn is_point_in_rect(&self, x: u16, y: u16, rect: &Rect) -> bool {
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }

    /// Get hover feedback for UI elements
    pub fn get_hover_info(&self, clickable_areas: &[ClickableArea]) -> Option<String> {
        if let Some((x, y)) = self.hover_position {
            for area in clickable_areas {
                if self.is_point_in_rect(x, y, &area.area) {
                    return Some(match &area.element_type {
                        ClickableElementType::Button => format!("Click to activate {}", area.id),
                        ClickableElementType::ListItem(index) => {
                            format!("Click to select item {}", index)
                        }
                        ClickableElementType::Battlefield => "Click to move/attack".to_string(),
                        ClickableElementType::MenuItem => format!("Click to select {}", area.id),
                        ClickableElementType::TabHeader => format!("Switch to {} tab", area.id),
                        ClickableElementType::CloseButton => "Click to close".to_string(),
                        ClickableElementType::Scroll => "Scroll to navigate".to_string(),
                    });
                }
            }
        }
        None
    }
}

/// Helper functions for creating clickable areas
impl ClickableArea {
    pub fn button(id: String, area: Rect) -> Self {
        Self {
            id,
            area,
            element_type: ClickableElementType::Button,
        }
    }

    pub fn list_item(id: String, area: Rect, index: usize) -> Self {
        Self {
            id,
            area,
            element_type: ClickableElementType::ListItem(index),
        }
    }

    pub fn battlefield(id: String, area: Rect) -> Self {
        Self {
            id,
            area,
            element_type: ClickableElementType::Battlefield,
        }
    }

    pub fn menu_item(id: String, area: Rect) -> Self {
        Self {
            id,
            area,
            element_type: ClickableElementType::MenuItem,
        }
    }

    pub fn close_button(area: Rect) -> Self {
        Self {
            id: "close".to_string(),
            area,
            element_type: ClickableElementType::CloseButton,
        }
    }
}

/// Battlefield-specific mouse handling
pub struct BattlefieldMouseHandler {
    pub tile_width: f32,
    pub tile_height: f32,
    pub battlefield_offset: (u16, u16),
}

impl BattlefieldMouseHandler {
    pub fn new() -> Self {
        Self {
            tile_width: 2.0,  // Tiles are typically 2 characters wide in terminal
            tile_height: 1.0, // Tiles are 1 row high
            battlefield_offset: (0, 0),
        }
    }

    /// Convert terminal coordinates to battlefield tile coordinates
    pub fn terminal_to_battlefield_coords(&self, term_x: u16, term_y: u16) -> Option<(i32, i32)> {
        if term_x < self.battlefield_offset.0 || term_y < self.battlefield_offset.1 {
            return None;
        }

        let relative_x = (term_x - self.battlefield_offset.0) as f32;
        let relative_y = (term_y - self.battlefield_offset.1) as f32;

        let battlefield_x = (relative_x / self.tile_width) as i32;
        let battlefield_y = (relative_y / self.tile_height) as i32;

        Some((battlefield_x, battlefield_y))
    }

    /// Set the battlefield area offset for coordinate conversion
    pub fn set_battlefield_area(&mut self, area: Rect) {
        self.battlefield_offset = (area.x, area.y);
    }
}

impl Default for MouseHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BattlefieldMouseHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for adding mouse support to UI states
pub trait MouseSupported {
    fn get_clickable_areas(&self) -> Vec<ClickableArea>;
    fn handle_mouse_interaction(&mut self, interaction: MouseInteraction) -> bool;
}