use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use gtk4::CssProvider;

const DEFAULT_CSS: &str = "
/* reffer to https://thomashunter.name/i3-configurator/ */

.focused_tab {
    background-color: #4C7899;
}

.tab {
    color: #FFFFFF;

    min-height: 20em;
    min-width: 35em;

    padding: 0.5em;
}

.window {
    background-color: #333333;
    border-style: solid;
    border-width: 2px;
    border-color: #4C7899;
}

.picture {

}

.tabs {
    background-color: #333333;
    margin: 1em;
}

.label {
    
}

";

pub fn init() {
    let provider = CssProvider::new();
    provider.load_from_data(&get_css());
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn get_css() -> String {
    let home_dir = dirs::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap();
    let scss_file = format!("{}/.i3-switcher-style.css", home_dir);
    let css_file_path = Path::new(&scss_file);

    if !css_file_path.exists() {
        let settings_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(css_file_path);

        settings_file
            .unwrap()
            .write(DEFAULT_CSS.as_bytes())
            .unwrap();
    }

    let css = fs::read_to_string(css_file_path).unwrap();

    return css;
}
