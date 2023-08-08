#[derive(Debug)]
pub enum ContentType {
    Text(String),
    Image(String),
    Autio(String),
    Video(String),
    Application(String),
    MultiPart { sub_type: String, boundary: String },
    None,
}

impl ContentType {
    pub fn parse(content_type: &str) -> Self {
        if content_type.is_empty() {
            return Self::None;
        }

        let mut boundary = String::new();
        let sub_type: String = if content_type.contains(";") {
            let v: Vec<&str> = content_type.split(";").collect();
            if content_type.contains("boundary=") {
                let v: Vec<&str> = content_type.split("boundary=").collect();
                boundary.push_str(v[1].trim());
            }
            let v: Vec<&str> = v[0].split("/").collect();
            v[1].trim().to_string()
        } else {
            let v: Vec<&str> = content_type.split("/").collect();
            v[1].trim().to_string()
        };

        if content_type.starts_with("text") {
            Self::Text(sub_type)
        } else if content_type.starts_with("image") {
            Self::Image(sub_type)
        } else if content_type.starts_with("audio") {
            Self::Autio(sub_type)
        } else if content_type.starts_with("video") {
            Self::Video(sub_type)
        } else if content_type.starts_with("application") {
            Self::Application(sub_type)
        } else if boundary != "" {
            Self::MultiPart { sub_type, boundary }
        } else {
            Self::None
        }
    }
}
