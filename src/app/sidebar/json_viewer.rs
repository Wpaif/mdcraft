use eframe::egui;

#[derive(Clone, Copy)]
enum JsonContainer {
    Object { expecting_key: bool },
    Array,
}

fn push_json_text(job: &mut egui::text::LayoutJob, text: &str, color: egui::Color32) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(13.0),
            color,
            ..Default::default()
        },
    );
}

pub(super) fn json_layout_job(ui: &egui::Ui, text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let default_color = ui.visuals().text_color();
    let punct_color = default_color.gamma_multiply(0.9);
    let key_color = ui.visuals().hyperlink_color;
    let string_color = default_color.gamma_multiply(0.95);
    let number_color = ui.visuals().warn_fg_color;
    let bool_color = egui::Color32::from_rgb(96, 197, 139);
    let null_color = ui.visuals().error_fg_color;

    let mut stack: Vec<JsonContainer> = Vec::new();
    let mut i = 0usize;
    let bytes = text.as_bytes();

    while i < bytes.len() {
        let ch = bytes[i] as char;

        if ch.is_whitespace() {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            push_json_text(&mut job, &text[start..i], default_color);
            continue;
        }

        match ch {
            '{' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.push(JsonContainer::Object {
                    expecting_key: true,
                });
                i += 1;
            }
            '}' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.pop();
                i += 1;
            }
            '[' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.push(JsonContainer::Array);
                i += 1;
            }
            ']' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                stack.pop();
                i += 1;
            }
            ':' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                if let Some(JsonContainer::Object { expecting_key }) = stack.last_mut() {
                    *expecting_key = false;
                }
                i += 1;
            }
            ',' => {
                push_json_text(&mut job, &text[i..i + 1], punct_color);
                if let Some(JsonContainer::Object { expecting_key }) = stack.last_mut() {
                    *expecting_key = true;
                }
                i += 1;
            }
            '"' => {
                let start = i;
                i += 1;
                let mut escaped = false;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }

                let is_key = matches!(
                    stack.last(),
                    Some(JsonContainer::Object {
                        expecting_key: true
                    })
                );
                let color = if is_key { key_color } else { string_color };
                push_json_text(&mut job, &text[start..i], color);
            }
            '-' | '0'..='9' => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-') {
                        i += 1;
                    } else {
                        break;
                    }
                }
                push_json_text(&mut job, &text[start..i], number_color);
            }
            't' if text[i..].starts_with("true") => {
                push_json_text(&mut job, "true", bool_color);
                i += 4;
            }
            'f' if text[i..].starts_with("false") => {
                push_json_text(&mut job, "false", bool_color);
                i += 5;
            }
            'n' if text[i..].starts_with("null") => {
                push_json_text(&mut job, "null", null_color);
                i += 4;
            }
            _ => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']' | ':' | ',' | '"') {
                        break;
                    }
                    i += 1;
                }
                push_json_text(&mut job, &text[start..i], default_color);
            }
        }
    }

    job
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::{json_layout_job, push_json_text};

    #[test]
    fn json_layout_job_and_push_json_text_generate_sections() {
        egui::__run_test_ui(|ui| {
            let job = json_layout_job(ui, "{\"a\":1,true:false,null:[1,2]}", 300.0);
            assert!(!job.sections.is_empty());

            let escaped = json_layout_job(ui, "{\"k\":\"a\\\\\"b\",x:[1,2],u:foo}", 300.0);
            assert!(!escaped.sections.is_empty());

            let punctuation_without_object = json_layout_job(ui, "[:,]", 300.0);
            assert!(!punctuation_without_object.sections.is_empty());

            let scientific_number = json_layout_job(ui, "{\"n\":1.2e-3}", 300.0);
            assert!(!scientific_number.sections.is_empty());

            let bare_identifier = json_layout_job(ui, "foobar", 300.0);
            assert!(!bare_identifier.sections.is_empty());

            let comma_outside = json_layout_job(ui, ",", 300.0);
            assert!(!comma_outside.sections.is_empty());
        });

        let mut job = egui::text::LayoutJob::default();
        push_json_text(&mut job, "", egui::Color32::WHITE);
        assert!(job.sections.is_empty());
        push_json_text(&mut job, "abc", egui::Color32::WHITE);
        assert_eq!(job.sections.len(), 1);
    }
}
