use gtk4::prelude::BoxExt;
use gtk4::prelude::WidgetExt;
use gtk4::Label;
use gtk4::Picture;
use xcap::image::ImageBuffer;
use xcap::image::Rgba;

use crate::screenshot;
use crate::state;
use crate::state::CURRENT_WS_NAME;

pub struct Tab {
    pub label: Label,
    pub gtk_box: gtk4::Box,
}

impl Tab {
    pub fn new(picture: Option<Picture>, name: Option<&str>) -> Self {
        let gtk_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
        gtk_box.add_css_class("tab");

        if let Some(ref pic) = picture {
            gtk_box.append(pic);
        }

        let label = Label::new(name);
        label.add_css_class("label");
        gtk_box.append(&label);

        Tab {
            label: Label::new(name),
            gtk_box: gtk_box,
        }
    }
}

pub struct TabsList {
    pub tabs_box: gtk4::Box,
    pub tabs_vec: Vec<Tab>,
}

impl TabsList {
    pub fn new() -> TabsList {
        let mut tabs = TabsList {
            tabs_box: gtk4::Box::new(gtk4::Orientation::Horizontal, 3),
            tabs_vec: Vec::<Tab>::new(),
        };

        tabs.tabs_box.set_homogeneous(true);
        tabs.tabs_box.add_css_class("tabs");

        let mut i3_conn_lock = state::I3_CONNECTION.write().unwrap();
        let wks = i3_conn_lock.get_workspaces().unwrap().workspaces;
        for (_, ws) in (&wks).iter().enumerate() {
            tabs.add_new_tab(None, &ws.name);
        }

        tabs.re_render();

        tabs
    }

    pub fn remove_tab(self: &mut Self, name: &String) {
        for (index, tab) in self.tabs_vec.iter().enumerate() {
            if tab.label.text().eq(name) {
                self.tabs_vec.remove(index);
                return;
            }
        }
    }

    pub fn add_new_tab(self: &mut Self, picture: Option<Picture>, name: &String) {
        let tab = Tab::new(picture, Some(name));
        self.tabs_vec.push(tab);
    }

    pub fn re_render(self: &mut Self) {
        loop {
            match self.tabs_box.first_child() {
                Some(tab) => self.tabs_box.remove(&tab),
                None => break,
            }
        }

        for tab in &self.tabs_vec {
            self.tabs_box.append(&tab.gtk_box);
        }
    }

    pub fn set_image(self: &mut Self, name: String, img: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>) {
        let mut i: Option<usize> = None;
        if let Some(pic) = img {
            for (index, tab) in self.tabs_vec.iter().enumerate() {
                if tab.label.text().eq(&name) {
                    i = Some(index);
                    break;
                }
            }

            if let Some(i) = i {
                let pixbuf = screenshot::rgba_image_to_pixbuf(&pic);
                let picture = Picture::for_pixbuf(&pixbuf);
                self.tabs_vec[i] = Tab::new(Some(picture), Some(&name));
            }

            self.re_render();
        }
    }

    pub fn reorder_prev_first(self: &mut Self) {
        if self.tabs_vec.len() < 2 {
            return;
        }

        if let Some(ref name) = *CURRENT_WS_NAME.read().unwrap() {
            for (index, tab) in self.tabs_vec.iter().enumerate() {
                if tab.label.text().eq(name) {
                    let tab = self.tabs_vec.remove(index as usize);
                    self.tabs_vec.insert(0, tab);
                    self.re_render();
                    return;
                }
            }
        }
    }
}
