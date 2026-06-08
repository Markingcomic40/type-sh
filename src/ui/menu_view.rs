use std::fmt::format;

use crate::{
    core::menu::{
        MenuSection, MenuState, BOOLEAN_OPTIONS, BUILTIN_WORDLISTS, TIMED_PRESETS, WORDS_PRESETS,
    },
    ui::theme::BUILTIN_THEMES,
};

pub struct SectionInput {
    pub buffer: String,
    pub placeholder: &'static str,
}

/// Layout data for a single menu section row
pub struct SectionLayout {
    pub label: &'static str,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub is_focused: bool,
    pub is_hidden: bool,
    pub active_input: Option<SectionInput>,
}

pub struct MenuView {
    pub sections: Vec<SectionLayout>,
    pub error_message: Option<&'static str>,
    pub is_inputting: bool,
}

pub fn compute_menu_layout(menu: &MenuState) -> MenuView {
    let mut sections = vec![
        mode_layout(menu),
        duration_layout(menu),
        wordlist_layout(menu),
        theme_layout(menu),
        freedom_layout(menu),
    ];

    if menu.is_inputting() {
        for section in &mut sections {
            if section.is_focused {
                section.active_input = Some(SectionInput {
                    buffer: menu.input_buffer().to_string(),
                    placeholder: placeholder_for(menu.focused_section),
                })
            }
        }
    }

    MenuView {
        sections,
        error_message: menu.error_message(),
        is_inputting: menu.is_inputting(),
    }
}

fn mode_layout(menu: &MenuState) -> SectionLayout {
    SectionLayout {
        label: "mode",
        options: vec!["timed".to_string(), "words".to_string(), "zen".to_string()],
        selected_index: menu.selected_mode,
        is_focused: menu.focused_section == MenuSection::Mode,
        is_hidden: false,
        active_input: None,
    }
}

fn duration_layout(menu: &MenuState) -> SectionLayout {
    let (label, options) = match menu.selected_mode {
        0 => {
            let mut options: Vec<String> = TIMED_PRESETS.iter().map(|p| format!("{p}s")).collect();
            options.push(custom_value_label(
                &menu.custom_duration.map(|t| format!("{t}s")),
            ));

            ("times", options)
        }
        1 => {
            let mut options: Vec<String> = WORDS_PRESETS.iter().map(|p| format!("{p}s")).collect();
            options.push(custom_value_label(
                &menu.custom_duration.map(|n| n.to_string()),
            ));

            ("words", options)
        }
        _ => ("time", vec![]),
    };

    SectionLayout {
        label: label,
        options: options,
        selected_index: menu.selected_duration,
        is_focused: menu.focused_section == MenuSection::Duration,
        is_hidden: menu.selected_mode == 2,
        active_input: None,
    }
}

fn wordlist_layout(menu: &MenuState) -> SectionLayout {
    let mut options: Vec<String> = BUILTIN_WORDLISTS.iter().map(|w| w.to_string()).collect();
    options.push(custom_path_label(&menu.custom_wordlist));

    SectionLayout {
        label: "wordlist",
        options: options,
        selected_index: menu.selected_wordlist,
        is_focused: menu.focused_section == MenuSection::Wordlist,
        is_hidden: false,
        active_input: None,
    }
}

fn theme_layout(menu: &MenuState) -> SectionLayout {
    let mut options: Vec<String> = BUILTIN_THEMES
        .iter()
        .map(|option| option.to_string())
        .collect();
    options.push(custom_value_label(&menu.custom_theme));

    SectionLayout {
        label: "theme",
        options: options,
        selected_index: menu.selected_theme,
        is_focused: menu.focused_section == MenuSection::Theme,
        is_hidden: false,
        active_input: None,
    }
}

fn freedom_layout(menu: &MenuState) -> SectionLayout {
    SectionLayout {
        label: "freedom mode",
        options: BOOLEAN_OPTIONS
            .iter()
            .map(|option| option.to_string())
            .collect(),
        selected_index: menu.selected_freedom,
        is_focused: menu.focused_section == MenuSection::Freedom,
        is_hidden: false,
        active_input: None,
    }
}

fn custom_value_label(value: &Option<String>) -> String {
    match value {
        Some(v) => format!("custom: {}", v),
        None => "custom".to_string(),
    }
}

fn custom_path_label(path: &Option<String>) -> String {
    // If the input is valid then we display the thing otherwise we display custom

    match path {
        Some(p) => {
            let filename = std::path::Path::new(p)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(p.as_str());
            format!("custom: {}", filename)
        }
        None => "custom".to_string(),
    }
}

fn placeholder_for(section: MenuSection) -> &'static str {
    match section {
        MenuSection::Duration => "enter a number...",
        MenuSection::Wordlist | MenuSection::Theme => "enter file path...",
        _ => "...",
    }
}
