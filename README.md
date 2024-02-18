# i3wm Alt+Tab Workspace Switcher

The i3wm Alt+Tab Workspace Switcher is a tool designed to bring the familiar Alt+Tab window switching functionality to the i3 window manager environment. It aims to enhance productivity by enabling users to switch between workspaces efficiently and intuitively, mimicking the window switching feature found in traditional desktop environments like Windows.

<p align="center">
  <img src="demo.gif" />
</p>


## Installation

The i3wm Alt+Tab Workspace Switcher can be easily installed on any Debian-based Linux distribution. Simply download the latest `.deb` package from the [releases section](https://github.com/iondodon/i3-switcher-x11/releases) of our GitHub repository and install it using your package manager.

For a quick installation, you can use the following command in the terminal:

```bash
sudo dpkg -i path/to/downloaded/i3-switcher-x11.deb
```

After installing, add the following two lines in the `~/.config/i3/config` file. 
The first line will make the i3switcherX1 be a floating window.
The second line will start the i3switcherx11 when i3wm starts.

```bash
for_window [title="i3switcherX11"] floating enable
exec --no-startup-id i3-switcher-x11
```

## License

This project is licensed under the MIT License. For more information, please refer to the LICENSE file included in the repository.