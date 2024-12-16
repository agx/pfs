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
use glib_macros::{clone, Properties};
use gtk::{gio, glib, CompositeTemplate};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::sync::OnceLock;

use crate::{config::LOG_DOMAIN, file_selector::SortMode, grid_item::GridItem, util};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[template(resource = "/mobi/phosh/FileSelector/dir-view.ui")]
    #[properties(wrapper_type = super::DirView)]
    pub struct DirView {
        #[template_child]
        pub grid_view: TemplateChild<gtk::GridView>,

        #[template_child]
        pub view_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub directory_list: TemplateChild<gtk::DirectoryList>,

        #[template_child]
        pub sorted_list: TemplateChild<gtk::SortListModel>,

        #[template_child]
        pub filtered_list: TemplateChild<gtk::FilterListModel>,

        #[template_child]
        pub single_selection: TemplateChild<gtk::SingleSelection>,

        #[template_child]
        pub item_factory: TemplateChild<gtk::SignalListItemFactory>,

        // The folder to display
        #[property(get, set = Self::set_folder, explicit_notify)]
        folder: RefCell<Option<gio::File>>,

        // `true` if there's a selected item
        #[property(get, explicit_notify)]
        pub(super) has_selection: Cell<bool>,

        // Icon size of the items in the grid view
        #[property(get, set)]
        icon_size: Cell<u32>,

        // What to sort for
        #[property(get, set = Self::set_sort_mode, builder(SortMode::default()))]
        pub sort_mode: RefCell<SortMode>,

        // Whether sort is reversed
        #[property(get, set = Self::set_reversed, explicit_notify)]
        pub(super) reversed: Cell<bool>,

        // Whether to sort directories before files
        #[property(get, set)]
        pub(super) directories_first: Cell<bool>,

        // Whether to show hidden files
        #[property(get, set, set = Self::set_show_hidden, explicit_notify)]
        pub(super) show_hidden: Cell<bool>,

        // Whether to select a directory rather than a file
        #[property(get, set = Self::set_directories_only, explicit_notify)]
        pub(super) directories_only: Cell<bool>,

        // The current filter type filter
        #[property(get, set = Self::set_type_filter, construct, nullable, explicit_notify)]
        pub(super) type_filter: RefCell<Option<gtk::FileFilter>>,

        // The current filter type filter plus directories
        #[property(get, set = Self::set_type_filter, nullable, explicit_notify)]
        pub(super) real_filter: RefCell<Option<gtk::FileFilter>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DirView {
        const NAME: &'static str = "PfsDirView";
        type Type = super::DirView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl DirView {
        // r/o property
        pub(super) fn set_has_selection(&self, has_selection: bool) {
            if has_selection == self.has_selection.get() {
                return;
            }

            self.has_selection.replace(has_selection);
            self.obj().notify_has_selection();
        }

        fn update_directory_selection(&self) {
            // In directory selection mode we have a selection whenever
            // we're in a valid dir (e.g. not in recent:///
            if !self.directories_only.get() {
                return;
            }

            let has_selection = util::is_valid_folder(&self.folder.borrow());
            self.set_has_selection(has_selection);
        }

        fn set_folder(&self, folder: Option<gio::File>) {
            let obj = self.obj();
            let oldfolder = self.folder.borrow().clone();

            if folder.is_none() {
                return;
            }

            let folder = folder.unwrap();
            if oldfolder.is_some() && folder.equal(&oldfolder.unwrap()) {
                return;
            }

            let uri = folder.uri();
            glib::g_debug!(LOG_DOMAIN, "Loading folder for {uri:#?}");

            *self.folder.borrow_mut() = Some(folder);
            obj.notify_folder();

            self.update_directory_selection();
        }

        fn set_show_hidden(&self, show_hidden: bool) {
            let obj = self.obj();

            if self.show_hidden.get() == show_hidden {
                return;
            }

            glib::g_debug!(LOG_DOMAIN, "show_hidden {show_hidden:#?}");

            self.show_hidden.replace(show_hidden);
            obj.notify_show_hidden();

            // Refilter
            let filter = self.filtered_list.filter().unwrap();
            let strict = match show_hidden {
                true => gtk::FilterChange::LessStrict,
                false => gtk::FilterChange::MoreStrict,
            };
            filter.emit_by_name::<()>("changed", &[&strict]);
        }

        fn set_sort_mode(&self, mode: SortMode) {
            if *self.sort_mode.borrow() == mode {
                return;
            }

            let reversed = self.reversed.get();
            self.obj().set_sorting(mode, reversed);
        }

        fn set_reversed(&self, reversed: bool) {
            if self.reversed.get() == reversed {
                return;
            }

            let mode = *self.sort_mode.borrow();
            self.obj().set_sorting(mode, reversed);
        }

        fn set_directories_only(&self, directories_only: bool) {
            let obj = self.obj();

            if self.directories_only.get() == directories_only {
                return;
            }

            glib::g_debug!(LOG_DOMAIN, "directories_only {directories_only:#?}");

            self.directories_only.replace(directories_only);

            // Refilter
            let filter = self.filtered_list.filter().unwrap();
            let strict = match directories_only {
                false => gtk::FilterChange::LessStrict,
                true => gtk::FilterChange::MoreStrict,
            };
            filter.emit_by_name::<()>("changed", &[&strict]);

            obj.notify_directories_only();
            self.update_directory_selection();
        }

        fn set_type_filter(&self, type_filter: Option<gtk::FileFilter>) {
            let obj = self.obj();

            if *self.type_filter.borrow() == type_filter {
                return;
            }

            // Ensure directories are always included in the filter so users's can browse
            // through them. We don't modify the passed in filter as the user might read
            // it back
            if type_filter.is_some() {
                let filter = type_filter.clone().unwrap();
                let real_filter = gtk::FileFilter::from_gvariant(&filter.to_gvariant());
                real_filter.add_mime_type("inode/directory");

                let name = real_filter.name().unwrap_or_default();
                glib::g_debug!(LOG_DOMAIN, "Setting file filter to {name:#?}");
                *self.real_filter.borrow_mut() = Some(real_filter);
            } else {
                *self.real_filter.borrow_mut() = None;
                glib::g_debug!(LOG_DOMAIN, "Setting file filter to None");
            }

            *self.type_filter.borrow_mut() = type_filter;
            obj.notify_type_filter();
            obj.notify_real_filter();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DirView {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_icon_size(96);
            obj.set_directories_first(true);

            obj.setup_sort_and_filter();

            obj.bind_property("folder", &self.directory_list.get(), "file")
                .sync_create()
                .build();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("new-uri")
                        .param_types([String::static_type()])
                        .build(),
                    // The UI should consider updating the displayed
                    // filename
                    Signal::builder("new-filename")
                        .param_types([String::static_type()])
                        .build(),
                ]
            })
        }
    }

    impl WidgetImpl for DirView {}
    impl BinImpl for DirView {}
}

glib::wrapper! {
    pub struct DirView(ObjectSubclass<imp::DirView>)
        @extends adw::Bin, gtk::Widget;
}

impl Default for DirView {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl DirView {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_directory(&self, fileinfo: &gio::FileInfo) -> bool {
        let content_type = fileinfo.content_type().unwrap_or_default();

        if content_type == "inode/directory" {
            return true;
        }
        false
    }

    #[template_callback]
    fn on_item_setup(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let grid_item = GridItem::new();

        self.bind_property("icon-size", &grid_item, "icon-size")
            .sync_create()
            .build();

        list_item.set_child(Some(&grid_item));
    }

    #[template_callback]
    fn on_item_bind(&self, object: glib::Object) {
        let list_item = object.downcast_ref::<gtk::ListItem>().unwrap();
        let item = list_item.item().unwrap();
        let info = item.downcast_ref::<gio::FileInfo>().unwrap();

        let widget = list_item.child().unwrap();
        let grid_item = widget.downcast_ref::<GridItem>().unwrap();

        grid_item.set_fileinfo(info);
    }

    #[template_callback]
    fn on_selection_changed(&self, position: u32, n_items: u32) {
        glib::g_debug!(LOG_DOMAIN, "Selection changed {position:#?} {n_items:#?}");

        let selection = self.imp().single_selection.get();
        let selected_item = selection.selected_item();
        let mut is_selected = false;

        if selected_item.is_some() {
            let info = selected_item.unwrap();
            let fileinfo = info.downcast_ref::<gio::FileInfo>().unwrap();
            let object = fileinfo.attribute_object("standard::file").unwrap();
            let file = object.downcast_ref::<gio::File>().unwrap();

            if self.is_directory(fileinfo) {
                let uri = file.uri();

                glib::g_debug!(LOG_DOMAIN, "Should open {uri:#?}");
                self.imp().obj().emit_by_name::<()>("new-uri", &[&uri]);
            } else {
                is_selected = true;
                let filename = file.basename();
                self.imp()
                    .obj()
                    .emit_by_name::<()>("new-filename", &[&filename]);
            }
        }

        if self.directories_only() {
            return;
        }

        self.imp().set_has_selection(is_selected);
    }

    #[template_callback]
    fn on_n_items_changed(&self) {
        let n_items = self.imp().filtered_list.get().n_items();
        let pagename = if n_items > 0 { "folder" } else { "empty" };
        self.imp().view_stack.get().set_visible_child_name(pagename);
    }

    #[template_callback]
    fn on_activate(&self, pos: u32) {
        glib::g_debug!(LOG_DOMAIN, "Item Activated {pos:#?}");

        self.imp().single_selection.set_selected(pos);
        // Only accept when we have a selection
        if !self.has_selection() {
            return;
        }

        let _ = self
            .upcast_ref::<gtk::Widget>()
            .activate_action("file-selector.accept", None);
    }

    pub fn selected(&self) -> Option<Vec<String>> {
        let vec = if self.directories_only() {
            match self.folder().unwrap().path() {
                None => return None,
                Some(_) => vec![self.folder().unwrap().uri().to_string()],
            }
        } else {
            let selected = self.imp().single_selection.get().selected_item();
            let item = match selected {
                None => return None,
                Some(item) => item,
            };

            let file = item
                .downcast_ref::<gio::FileInfo>()
                .unwrap()
                .attribute_object("standard::file")
                .unwrap();

            let uri = file.downcast_ref::<gio::File>().unwrap().uri();
            glib::g_debug!(LOG_DOMAIN, "Uri {uri:#?}");

            vec![uri.to_string()]
        };
        Some(vec)
    }

    fn sort_by_name(&self, info1: &gio::FileInfo, info2: &gio::FileInfo) -> gtk::Ordering {
        match info1.display_name().cmp(&info2.display_name()) {
            Ordering::Less => {
                if self.imp().reversed.get() {
                    return gtk::Ordering::Larger;
                } else {
                    return gtk::Ordering::Smaller;
                }
            }
            Ordering::Greater => {
                if self.imp().reversed.get() {
                    return gtk::Ordering::Smaller;
                } else {
                    return gtk::Ordering::Larger;
                }
            }
            Ordering::Equal => gtk::Ordering::Equal,
        }
    }

    fn sort_by_modification_time(
        &self,
        info1: &gio::FileInfo,
        info2: &gio::FileInfo,
    ) -> gtk::Ordering {
        match info1
            .modification_date_time()
            .cmp(&info2.modification_date_time())
        {
            Ordering::Less => {
                if self.imp().reversed.get() {
                    return gtk::Ordering::Larger;
                } else {
                    return gtk::Ordering::Smaller;
                }
            }
            Ordering::Greater => {
                if self.imp().reversed.get() {
                    return gtk::Ordering::Smaller;
                } else {
                    return gtk::Ordering::Larger;
                }
            }
            Ordering::Equal => gtk::Ordering::Equal,
        }
    }

    fn setup_sort_and_filter(&self) {
        let sorter = gtk::CustomSorter::new(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            gtk::Ordering::Equal,
            move |obj1, obj2| {
                let info1 = obj1
                    .downcast_ref::<gio::FileInfo>()
                    .expect("Should be file info");
                let info2 = obj2
                    .downcast_ref::<gio::FileInfo>()
                    .expect("Should be file info");

                if this.directories_first() {
                    let is_dir1 = this.is_directory(info1);
                    let is_dir2 = this.is_directory(info2);

                    if is_dir1 && !is_dir2 {
                        return gtk::Ordering::Smaller;
                    }

                    if is_dir2 && !is_dir1 {
                        return gtk::Ordering::Larger;
                    }
                }

                let mode = *this.imp().sort_mode.borrow();
                match mode {
                    SortMode::DisplayName => this.sort_by_name(&info1, &info2),
                    SortMode::ModificationTime => this.sort_by_modification_time(&info1, &info2),
                }
            }
        ));
        self.imp().sorted_list.set_sorter(Some(&sorter));

        let custom_filter = gtk::CustomFilter::new(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            true,
            move |obj| {
                let info = obj
                    .downcast_ref::<gio::FileInfo>()
                    .expect("Should be file info");

                if this.imp().directories_only.get() && !this.is_directory(info) {
                    return false;
                }

                if this.imp().show_hidden.get() {
                    return true;
                }

                if info.display_name().starts_with('.') {
                    return false;
                }
                return true;
            }
        ));
        self.imp().filtered_list.set_filter(Some(&custom_filter));
    }

    pub fn set_sorting(&self, sort_mode: SortMode, reversed: bool) {
        glib::g_debug!(
            LOG_DOMAIN,
            "Sorting mode {sort_mode:#?}, reversed: {reversed:#?}"
        );

        *self.imp().sort_mode.borrow_mut() = sort_mode;
        self.imp().reversed.replace(reversed);

        self.notify_sort_mode();
        self.notify_reversed();

        // Resort
        let sorter = self.imp().sorted_list.sorter().unwrap();
        let change = gtk::SorterChange::Inverted;
        sorter.emit_by_name::<()>("changed", &[&change]);
    }
}
