fn main() {
    println!("cargo:rustc-link-lib=X11");
}

// needed: 
// libgtk-3-dev for gtk
// libappindicator3-1, libappindicator3-dev, libayatana-appindicator3-dev for systray-dev
// llvm clang libclang-dev  
// libx11-dev
