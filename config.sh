#!/bin/bash

# Define the floating rule
FLOATING_RULE='for_window [title="First GTK Program"] floating enable'

# Path to the i3 config file
I3_CONFIG="$HOME/.config/i3/config"

# Check if i3 config exists
if [ -f "$I3_CONFIG" ]; then
    # Append the floating rule if it's not already in the config
    if ! grep -qF "$FLOATING_RULE" "$I3_CONFIG"; then
        echo "Appending floating rule to your i3 config..."
        echo "$FLOATING_RULE" >> "$I3_CONFIG"
        # Reload i3 configuration to apply changes
        i3-msg reload
    else
        echo "Floating rule already exists in your i3 config."
    fi
else
    echo "i3 config not found at $I3_CONFIG."
fi
