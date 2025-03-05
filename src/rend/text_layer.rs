// TODO: create ability to reserve extra.

use std::rc::Rc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct TextLayer {
    pub(super) buffer: glyphon::Buffer,
    font_system: Rc<Mutex<glyphon::FontSystem>>,
    name: &'static str,
    pub attrs: glyphon::Attrs<'static>,
    pub shaping: glyphon::Shaping,
    pub left: f32,
    pub top: f32,
    pub scale: f32,
    pub bounds: glyphon::TextBounds,
    pub default_color: glyphon::Color,
    pub custom_glyphs: Vec<glyphon::CustomGlyph>,
}

#[derive(Default)]
pub struct TextLayerDesc {
    pub name: &'static str,
    pub attrs: Option<glyphon::Attrs<'static>>,
    pub shaping: Option<glyphon::Shaping>,
    pub left: f32,
    pub top: f32,
    pub scale: f32,
    pub bounds: glyphon::TextBounds,
    pub default_color: Option<glyphon::Color>,
    pub custom_glyphs: Vec<glyphon::CustomGlyph>,
}

impl TextLayer {
    pub fn new(
        buffer: glyphon::Buffer,
        desc: TextLayerDesc,
        font_system: Rc<Mutex<glyphon::FontSystem>>,
    ) -> Self {
        Self {
            font_system,
            buffer,
            attrs: desc.attrs.unwrap_or_else(glyphon::Attrs::new),
            shaping: desc.shaping.unwrap_or(glyphon::Shaping::Basic),
            name: desc.name,
            left: desc.left,
            top: desc.top,
            scale: desc.scale,
            bounds: desc.bounds,
            custom_glyphs: desc.custom_glyphs,
            default_color: desc.default_color.unwrap_or(glyphon::Color(0)),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn to_area(&self) -> glyphon::TextArea<'_> {
        glyphon::TextArea {
            buffer: &self.buffer,
            left: self.left,
            top: self.top,
            scale: self.scale,
            bounds: self.bounds,
            default_color: self.default_color,
            custom_glyphs: &self.custom_glyphs,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.buffer.set_text(
            &mut self.font_system.lock().unwrap(),
            text,
            self.attrs,
            self.shaping,
        );
    }
}
