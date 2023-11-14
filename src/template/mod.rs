use crate::util;

pub fn create(template: Option<&String>) -> String {
    let template = match template {
        None => return String::new(),
        Some(tmp) => tmp,
    };

    let items = vec![("{{DATE}}", util::get_date())];

    let mut text = template.to_string();
    for (placeholder, s) in items {
        text = text.replace(placeholder, &s);
    }

    text
}
