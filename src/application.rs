/*
 * Copyright 2024 The Phosh Developers
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Author: Guido Günther <agx@sigxcpu.org>
 */

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::PfsDemoWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PfsApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for PfsApplication {
        const NAME: &'static str = "PfsApplication";
        type Type = super::PfsApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for PfsApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for PfsApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = application.active_window().unwrap_or_else(|| {
                let window = PfsDemoWindow::new(&*application);
                window.upcast()
            });

            window.present();
        }
    }

    impl GtkApplicationImpl for PfsApplication {}
    impl AdwApplicationImpl for PfsApplication {}
}

glib::wrapper! {
    pub struct PfsApplication(ObjectSubclass<imp::PfsApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PfsApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("pfs")
            .application_icon("mobi.phosh.Pfs")
            .developer_name("Guido Günther")
            .version(VERSION)
            .developers(vec!["Guido Günther"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an email or URL.
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2024 The Phosh Developers")
            .build();

        about.present(Some(&window));
    }
}
