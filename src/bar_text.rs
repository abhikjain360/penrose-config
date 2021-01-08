use penrose::draw::{bar::widgets::Text, XCBDraw, TextStyle};
use penrose::Result;

use crate::FONT;
use crate::colors;
use std::time::*;

// pub fn time_text() -> Result<Box<Text>> {
//     Ok(Box::new(Text::new(

//         Box::new(XCBDraw::new()?),
//         &TextStyle {
//             font: FONT.into(),
//             point_size: 14,
//             fg: colors::WHITE.into(),
//             bg: Some(colors::BLACK.into()),
//             padding: (2.5, 2.5)
//         },
//         false,
//         false,
//     )))
// }
