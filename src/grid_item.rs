/*
 * Copyright 2024 The Phosh Developers
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Author: Guido Günther <agx@sigxcpu.org>
 */

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib, CompositeTemplate};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/mobi/phosh/FileSelector/grid-item.ui")]
    #[properties(wrapper_type = super::GridItem)]
    pub struct GridItem {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        #[property(get, set = Self::set_fileinfo)]
        fileinfo: RefCell<Option<gio::FileInfo>>,

        #[property(get, set)]
        icon_size: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GridItem {
        const NAME: &'static str = "PfsGridItem";
        type Type = super::GridItem;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl GridItem {
        fn set_fileinfo(&self, info: gio::FileInfo) {
            self.label.get().set_label(&info.display_name());
            if let Some(icon) = info.icon() {
                self.icon.get().set_from_gicon(&icon)
            }
            *self.fileinfo.borrow_mut() = Some(info);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for GridItem {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_icon_size(32);
        }
    }

    impl WidgetImpl for GridItem {}
    impl BinImpl for GridItem {}
}

glib::wrapper! {
    pub struct GridItem(ObjectSubclass<imp::GridItem>)
        @extends adw::Bin, gtk::Widget;
}

impl Default for GridItem {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl GridItem {
    pub fn new() -> Self {
        Self::default()
    }
}
