use gtk4::CssProvider;

/// reffer to https://thomashunter.name/i3-configurator/
pub fn init() {
    let provider = CssProvider::new();
    provider.load_from_data(
        "
        .selected_tab {
            background-color: #4C7899;
        }

        .tab {
            color: #FFFFFF;
        }

        .window {
            background-color: #333333;
            border-style: solid;
            border-width: 2px;
            border-color: #4C7899;
        }

        .picture {
            margin-top: 4px;
        }

        .tabs {
            background-color: #333333;
            margin: 5px;
            padding: 0.3px;
        }
    ",
    );
    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
