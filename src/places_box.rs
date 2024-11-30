/*
 * Copyright 2024 The Phosh Developers
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Author: Guido Günther <agx@sigxcpu.org>
 */

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use glib::Object;
use gtk::{gio, glib, CompositeTemplate};
use std::sync::OnceLock;

use crate::{config::LOG_DOMAIN, places_item::PlacesItem, util};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/mobi/phosh/FileSelector/places-box.ui")]
    pub struct PlacesBox {
        #[template_child]
        pub flow_box: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlacesBox {
        const NAME: &'static str = "PfsPlacesBox";
        type Type = super::PlacesBox;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlacesBox {
        fn constructed(&self) {
            self.parent_constructed();

            let item = Object::builder::<PlacesItem>()
                .property("place", &gettextrs::gettext("Recent"))
                .property("icon-name", "document-open-recent-symbolic")
                .property("uri", "recent:///")
                .build();
            self.flow_box.append(&item);

            let home = gio::File::for_path(&glib::home_dir());
            let item = Object::builder::<PlacesItem>()
                .property("place", &gettextrs::gettext("Home"))
                .property("icon-name", "user-home-symbolic")
                .property("uri", home.uri())
                .build();
            self.flow_box.append(&item);

            let home = gio::File::for_path(&glib::home_dir());
            for (dir, icon) in util::SPECIAL_DIRS.iter() {
                let Some(path) = glib::user_special_dir(*dir) else {
                    continue;
                };
                let folder = gio::File::for_path(&path);

                if folder.equal(&home) {
                    continue;
                }

                let name = path.file_name().unwrap();
                let item = Object::builder::<PlacesItem>()
                    .property("place", name.to_str())
                    .property("icon-name", icon)
                    .property("uri", folder.uri())
                    .build();
                self.flow_box.append(&item);
            }

            let item = Object::builder::<PlacesItem>()
                .property("place", &gettextrs::gettext("Trash"))
                .property("icon-name", "user-trash-symbolic")
                .property("uri", "trash:///")
                .build();
            self.flow_box.append(&item);

            // TODO: mounts, bookmarks, other locations
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("new-uri")
                    .param_types([String::static_type()])
                    .build()]
            })
        }
    }

    impl PlacesBox {}
    impl WidgetImpl for PlacesBox {}
    impl BinImpl for PlacesBox {}
}

glib::wrapper! {
    pub struct PlacesBox(ObjectSubclass<imp::PlacesBox>)
        @extends adw::Bin, gtk::Widget;
}

impl Default for PlacesBox {
    fn default() -> Self {
        glib::Object::new::<Self>(/*&[]*/)
    }
}

#[gtk::template_callbacks]
impl PlacesBox {
    pub fn new() -> Self {
        Self::default()
    }

    #[template_callback]
    fn on_item_activated(&self, flowboxchild: gtk::FlowBoxChild) {
        let object = flowboxchild.child().unwrap();
        let item = object.downcast_ref::<PlacesItem>().unwrap();

        let uri: String = item.uri();
        glib::g_debug!(LOG_DOMAIN, "Should open {uri:#?}");
        self.imp().obj().emit_by_name::<()>("new-uri", &[&uri]);
    }
}
