fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xrandr");
}

// needed: 
// libgtk-4-dev for gtk
// libappindicator3-1, libappindicator3-dev, libayatana-appindicator3-dev for systray-dev
// llvm clang libclang-dev  
// libx11-dev
// libxcb-randr0-dev, libxrandr-dev for screenshots