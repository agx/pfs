/*
 * Copyright 2025 The Phosh Developers
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Author: Guido Günther <agx@sigxcpu.org>
 */

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::process::Command;

use pfs::file_selector::{FileSelector, FileSelectorMode};

use crate::config::LOG_DOMAIN;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PfsOpenApplication {
        pub hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PfsOpenApplication {
        const NAME: &'static str = "PfsOpenApplication";
        type Type = super::PfsOpenApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for PfsOpenApplication {}

    impl ApplicationImpl for PfsOpenApplication {
        fn activate(&self) {
            let application = self.obj();

            *self.hold_guard.borrow_mut() = Some(application.hold());

            let home = glib::home_dir();
            let file_selector = glib::Object::builder::<FileSelector>()
                .property("accept_label", "Done")
                .property("title", "Select a File")
                .property("current-folder", gio::File::for_path(&home))
                .build();

            file_selector.connect_closure(
                "done",
                false,
                glib::closure_local!(
                    #[weak(rename_to = this)]
                    self,
                    move |selector: FileSelector, success: bool| {
                        glib::g_debug!(LOG_DOMAIN, "File dialog done, result: {success:#?}");
                        let selected = selector.selected();

                        if success {
                            let uris = match selected {
                                None => vec!["".to_string()],
                                Some(vec) => vec,
                            };
                            glib::g_message!(LOG_DOMAIN, "Opening {uris:#?}");

                            Command::new("gio")
                                .arg("open")
                                .arg(&uris[0])
                                .spawn()
                                .expect("Failed to open {uris[0]:?}");
                        }
                        // Drop the application ref count
                        this.hold_guard.replace(None);
                    }
                ),
            );

            file_selector.set_mode(FileSelectorMode::OpenFile);
            file_selector.present();
        }
    }

    impl GtkApplicationImpl for PfsOpenApplication {}
    impl AdwApplicationImpl for PfsOpenApplication {}
}

glib::wrapper! {
    pub struct PfsOpenApplication(ObjectSubclass<imp::PfsOpenApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PfsOpenApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }
}
