use crossterm::style::Color;

use crate::editor::annotated::annotation::AnnotationType;

pub struct Attribute {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl From<AnnotationType> for Attribute {
    fn from(value: AnnotationType) -> Self {
        match value {
            AnnotationType::Match => Self {
                foreground: Some(Color::White),
                background: Some(Color::Rgb {
                    r: 100,
                    g: 100,
                    b: 100,
                }),
            },
            AnnotationType::SelectedMatch => Self {
                foreground: Some(Color::White),
                background: Some(Color::Rgb {
                    r: 255,
                    g: 251,
                    b: 0,
                }),
            },
            AnnotationType::Digit => Self {
                foreground: Some(Color::White),
                background: Some(Color::Rgb {
                    r: 255,
                    g: 99,
                    b: 71,
                }),
            },
            AnnotationType::Comment => Self {
                foreground: Some(Color::White),
                background: Some(Color::Rgb {
                    r: 100,
                    g: 100,
                    b: 100,
                }),
            },
        }
    }
}
