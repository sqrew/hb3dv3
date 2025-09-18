use crate::graphics::Color;

#[derive(Debug, Clone, Copy)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn top_left() -> Self {
        Self { x: 10.0, y: 10.0 }
    }

    pub fn top_right(screen_width: f32) -> Self {
        Self { x: screen_width - 10.0, y: 10.0 }
    }

    pub fn bottom_left(screen_height: f32) -> Self {
        Self { x: 10.0, y: screen_height - 30.0 }
    }

    pub fn bottom_right(screen_width: f32, screen_height: f32) -> Self {
        Self { x: screen_width - 10.0, y: screen_height - 30.0 }
    }

    pub fn center(screen_width: f32, screen_height: f32) -> Self {
        Self {
            x: screen_width / 2.0,
            y: screen_height / 2.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_size: u32,
    pub color: Color,
    pub align_h: HorizontalAlign,
    pub align_v: VerticalAlign,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16,
            color: Color::WHITE,
            align_h: HorizontalAlign::Left,
            align_v: VerticalAlign::Top,
        }
    }
}

impl TextStyle {
    pub fn new(font_size: u32, color: Color) -> Self {
        Self {
            font_size,
            color,
            ..Default::default()
        }
    }

    pub fn with_alignment(mut self, h_align: HorizontalAlign, v_align: VerticalAlign) -> Self {
        self.align_h = h_align;
        self.align_v = v_align;
        self
    }
}

#[derive(Debug, Clone)]
pub struct UIElement {
    pub text: String,
    pub position: Position,
    pub style: TextStyle,
}

impl UIElement {
    pub fn new(text: impl Into<String>, position: Position, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            position,
            style,
        }
    }

    pub fn fps_counter(fps: f32, screen_width: f32) -> Self {
        Self {
            text: format!("FPS: {:.1}", fps),
            position: Position::top_right(screen_width),
            style: TextStyle::new(18, Color::YELLOW).with_alignment(
                HorizontalAlign::Right,
                VerticalAlign::Top
            ),
        }
    }

    pub fn debug_info(text: impl Into<String>, position: Position) -> Self {
        Self {
            text: text.into(),
            position,
            style: TextStyle::new(14, Color::GREEN),
        }
    }
}

pub struct UIManager {
    elements: Vec<UIElement>,
    screen_width: f32,
    screen_height: f32,
}

impl UIManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            elements: Vec::new(),
            screen_width,
            screen_height,
        }
    }

    pub fn add_element(&mut self, element: UIElement) {
        self.elements.push(element);
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn update_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn elements(&self) -> &[UIElement] {
        &self.elements
    }

    // Helper methods for common UI elements
    pub fn add_fps_counter(&mut self, fps: f32) {
        self.add_element(UIElement::fps_counter(fps, self.screen_width));
    }

    pub fn add_debug_text(&mut self, text: impl Into<String>) {
        self.add_element(UIElement::debug_info(text, Position::top_left()));
    }

    pub fn add_centered_text(&mut self, text: impl Into<String>, style: TextStyle) {
        let position = Position::center(self.screen_width, self.screen_height);
        self.add_element(UIElement::new(text, position, style));
    }
}