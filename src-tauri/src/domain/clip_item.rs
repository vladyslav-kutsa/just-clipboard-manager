use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum ClipItem {
    Text(String),
    Image {
        hash: String,
        width: u32,
        height: u32,
    },
}

pub trait Preview {
    fn preview(&self) -> String;
}

impl Preview for ClipItem {
    fn preview(&self) -> String {
        match self {
            ClipItem::Text(i) => i.to_string(),
            ClipItem::Image { width, height, .. } => format!("image {}x{}", &width, &height),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn preview_text_test() {
        let clip_item = ClipItem::Text(String::from("hello"));
        assert_eq!(clip_item.preview(), "hello");
    }

    #[test]
    fn preview_image_test() {
        let clip_item = ClipItem::Image {
            hash: String::new(),
            width: 800,
            height: 600,
        };
        assert_eq!(clip_item.preview(), "image 800x600");
    }
}
