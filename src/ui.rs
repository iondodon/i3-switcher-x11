use gdk4::glib::{self, clone};
use gtk4::prelude::ButtonExt;
use gtk4::prelude::WidgetExt;
use gtk4::prelude::BoxExt;
use gtk4::Application;
use gtk4::{ApplicationWindow, Button, EventControllerKey};
use i3ipc::I3Connection;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use gtk4::prelude::GtkWindowExt;
use gtk4::glib::ControlFlow;


pub fn setup(app: &Application, i3_conn: Arc<Mutex<I3Connection>>, is_visible: Arc<AtomicBool>) {
    let window = ApplicationWindow::builder()
            .application(app)
            .title("First GTK Program")
            .default_width(350)
            .default_height(70)
            .build();

        let controller = EventControllerKey::new();
        let window_clone = window.clone();
        let is_visible_clone = is_visible.clone();
        controller.connect_key_released(move |_, keyval, _, _| {
            match keyval.name().unwrap().as_str() {
                "Alt_L" => { 
                    window_clone.hide(); 
                    println!("Alt released gtk");
                    is_visible_clone.store(false, Ordering::SeqCst);
                },
                _ => {}
            }
        });
        window.add_controller(controller);
        window.present();
        window.hide();

        update_window_content(&window, i3_conn.clone());

        let is_visible_clone = is_visible.clone();
        let i3_conn_clone = i3_conn.clone();
        glib::timeout_add_local(Duration::from_millis(100), clone!(@weak window => @default-return ControlFlow::Continue, move || {
            println!("Now is {}", is_visible_clone.load(Ordering::SeqCst));
            if is_visible_clone.load(Ordering::SeqCst) {
                update_window_content(&window, i3_conn_clone.to_owned());
                window.show();
            } else {
                window.hide();
            }
            glib::ControlFlow::Continue
        }));
}

fn update_window_content(window: &ApplicationWindow, i3_conn: Arc<Mutex<I3Connection>>) {
     let vbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 1);

     let mut i3_conn = i3_conn.lock().unwrap();
     let wks = i3_conn.get_workspaces().unwrap();

     for ws in &wks.workspaces {
         let button = Button::with_label(&ws.name);
         button.connect_clicked(|_| {
             eprintln!("Clicked!");
         });

         vbox.append(&button);
     }

     window.set_child(Some(&vbox));
}