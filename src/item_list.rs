use super::list_items::ListItems;
use super::utility::qs;
use crate::toolbar::ItemListModeToolbar;
pub use crate::traits::*;
use crate::utility::load_stylesheet;
use log;
use qt_core::{q_item_selection_model::SelectionFlag, Key, MatchFlag, QModelIndex, QString, Slot};
use qt_gui::{q_key_sequence::StandardKey, QKeySequence, QStandardItem, QStandardItemModel};
//use qt_widgets::cpp_core::Ref as QRef;
use qt_widgets::{
    cpp_core::Ref as QRef,
    cpp_core::{CppBox, MutPtr},
    q_abstract_item_view::DragDropMode,
    q_abstract_item_view::SelectionMode,
    q_action::ActionEvent,
    QComboBox, QFrame, QHBoxLayout, QLabel, QLayout, QListView, QPushButton, QShortcut, QWidget,
};
use std::cell::RefCell;
use std::rc::Rc;

#[allow(unused_macros)]
/// Macro to clone items before moving them into a closure.
/// Used to handle reference counted items without cluttering
/// the main code with a bunch of clone calls.
/// For closures, it looks like this:
/// ```ignore
/// enclose!{ (<CLONEME>,) move |<VARS>| {}}
///```
/// For Example
/// ```ignore
/// Slot::new(enclose!{(layout, toolbar) move || { ... do stuff }});
/// ```
macro_rules! enclose {
    ( ($(  $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[allow(unused_macros)]
macro_rules! take_ref {
    ( ($(  $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.as_ref();)*
            $y
        }
    };
}

#[allow(unused_macros)]
macro_rules! take_mut_ref {
    ( ($(  $x:ident ),*) $y:expr ) => {
        {
            #[allow(unused_mut)]
            $(let mut $x = $x.as_mut_ref();)*
            $y
        }
    };
}

#[allow(unused_macros)]
/// Works like enclose but provides for both non and mutable
/// clones.
///
/// # Example
/// ```ignore
/// { enclose_all! { (<ARG>) (mut <ARG>,) move |<ARG>| {} }}
/// ```
/// EG
/// ```ignore
/// Slot::new(enclose_all!{(layout) (mut toolbar, mut button)} move || {...do stuff});
/// ```
macro_rules! enclose_all {
    ( ($(  $x:ident ),*) ($( mut $mx:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            #[allow(unused_mut)]
            $(let mut $mx = $mx.clone();)*
            $y
        }
    };
}

//
// ITEMLIST
//
/// The ItemList provides a litview with a toolbar allowing you
/// to switch between reordering, deleting, and adding members.
/// It stores the main components that are interesting to
/// its clients, including the toolbar, the model, the view,
/// the actual items backing data, and various slots
pub struct ItemList<'l> {
    pub main: MutPtr<QWidget>,
    pub mode_toolbar: Rc<RefCell<ItemListModeToolbar>>,
    pub add_combobox: MutPtr<QComboBox>,
    pub model: CppBox<QStandardItemModel>,
    pub view: MutPtr<QListView>,
    pub items: Rc<RefCell<ListItems>>,
    pub enter_shortcut: MutPtr<QShortcut>,
    pub delete_shortcut: MutPtr<QShortcut>,
    pub cut_shortcut: MutPtr<QShortcut>,
    pub rm: Slot<'l>,
    pub find_mode: Slot<'l>,
    pub add_mode: Slot<'l>,
    pub enter_sc: Slot<'l>,
}

impl<'l> ItemList<'l> {
    /// New up an ItemList given a parent
    ///
    /// # Arguments
    /// * `parent` - MutPtr to the parent QWidget
    ///
    /// # Returns
    /// * An ItemList instance
    pub fn new(parent: &mut MutPtr<QWidget>) -> ItemList<'l> {
        unsafe {
            let mut main_ptr = Self::setup_main_widget(&parent);

            let mut model = Self::setup_model();
            let model_ptr = model.as_mut_ptr();

            let listitems = Rc::new(RefCell::new(ListItems::new(model_ptr)));

            let mode_toolbar = Rc::new(RefCell::new(ItemListModeToolbar::new(&mut main_ptr)));

            let (cblabel, cbox) = Self::setup_combobox("ItemCombo", &mut main_ptr);
            let cbox_ptr = cbox.clone();

            let listview_ptr = Self::setup_listview(model.as_mut_ptr(), &mut main_ptr.layout());

            let key_seq = QKeySequence::from_int(Key::KeyReturn.to_int());
            let enter_shortcut = QShortcut::new_2a(key_seq.as_ref(), main_ptr);
            //buttons
            let _save_button = Self::setup_button("Save", &mut main_ptr.layout());
            // shortcuts
            let key_seq = QKeySequence::from_int(Key::KeyBackspace.to_int());
            let delete_shortcut = QShortcut::new_2a(key_seq.as_ref(), main_ptr);

            let cut_key_seq = QKeySequence::from_standard_key(StandardKey::Cut);
            let cut_shortcut = QShortcut::new_2a(cut_key_seq.as_ref(), main_ptr);

            let rm_slot = Slot::new(enclose_all! { () (mut listview_ptr) move || {
                let selected = listview_ptr.selection_model().selected_indexes();
                if selected.length() == 0 {
                    return;
                }
                // we need to sort the indexes first. Otherwise, depending upon selection order, we
                // may not
                let mut indexes = (0..selected.size()).into_iter().map(|x| selected.at(x).row()).collect::<Vec<_>>();
                indexes.sort();
                indexes.iter().rev().for_each(|c| {listview_ptr.model().remove_row_1a(*c); });
            }});

            let enter_sc = Slot::new(
                enclose_all! { (mode_toolbar) (mut listitems, mut cbox_ptr, mut listview_ptr, mut model_ptr) move || {
                    let text = cbox_ptr.current_text();
                    if mode_toolbar.borrow().is_find_active() {
                        if Self::_scroll_to_item(text.as_ref(), &mut listview_ptr, &mut model_ptr, true) {
                            cbox_ptr.clear_edit_text();
                        }
                        return;
                    }
                    // bail if text is ""
                    if QString::compare_2_q_string(&text, &qs("")) == 0 {return;}
                    // validate that text is in the list
                    let mut found = false;
                    for cnt in 0..cbox_ptr.count() {
                        let item = cbox_ptr.item_text(cnt);
                        if QString::compare_2_q_string(&text,&item) == 0 {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        log::info!("user's entry not valid");
                        return;
                    }
                    if model_ptr.find_items_1a(&text).length() > 0 {
                        if Self::_scroll_to_item(text.as_ref(), &mut listview_ptr, &mut model_ptr, true) {
                            cbox_ptr.clear_edit_text();
                        }
                        return;
                    }

                    listitems.borrow_mut().add_item_to(text.to_std_string().as_str());//,&mut model_ptr);
                    cbox_ptr.clear_edit_text();
                    listview_ptr.scroll_to_bottom();

                }},
            );

            let f = Self {
                main: main_ptr,
                model,
                mode_toolbar,
                add_combobox: cbox,
                view: listview_ptr,
                items: listitems,
                enter_shortcut: enter_shortcut.into_ptr(),
                delete_shortcut: delete_shortcut.into_ptr(),
                cut_shortcut: cut_shortcut.into_ptr(),
                rm: rm_slot,

                find_mode: Slot::new(
                    take_mut_ref! { (cblabel) enclose_all! { () ( mut cbox_ptr) move || {
                        cbox_ptr.set_enabled(true);
                        if let Some(mut cblabel) = cblabel {cblabel.set_text(&qs("Find Item"))};
                    }}},
                ),

                add_mode: Slot::new(
                    take_mut_ref! {(cblabel) enclose_all! { () ( mut cbox_ptr) move || {
                        cbox_ptr.set_enabled(true);
                        if let Some(mut cblabel) = cblabel {cblabel.set_text(&qs("Add Item"))};

                    }}},
                ),

                enter_sc,
            };

            f.mode_toolbar
                .borrow_mut()
                .find_mode_action
                .triggered()
                .connect(&f.find_mode);

            f.mode_toolbar
                .borrow_mut()
                .add_mode_action
                .triggered()
                .connect(&f.add_mode);

            f.enter_shortcut.activated().connect(&f.enter_sc);

            f.delete_shortcut.activated().connect(&f.rm);

            f.cut_shortcut.activated().connect(&f.rm);

            f
        }
    }

    #[allow(dead_code)]
    /// Clear the listview and its backng model
    ///
    /// # Arguments
    /// None
    ///
    /// # Returns
    /// None
    pub fn clear(&mut self) {
        unsafe {
            self.items.borrow_mut().clear();
            self.model.clear();
        }
    }

    #[allow(dead_code)]
    /// Sets the contents to items, removing any pre-existing
    /// items.
    ///
    /// # Arguments
    /// * `items` - a Vector of &str or String
    ///
    /// # Returns
    /// None
    pub fn set_items<I>(&mut self, items: Vec<I>)
    where
        I: AsRef<str>,
    {
        unsafe {
            self.items.borrow_mut().clear();
            self.model.clear();
            for item in items {
                self.add_item(item.as_ref());
            }
        }
    }

    /// add an item to the pulldown
    ///
    /// # Arguments
    /// * The item to be added, as a &str or String
    pub fn add_item<I>(&mut self, item: I)
    where
        I: AsRef<str>,
    {
        self.items.borrow_mut().add_item_to(item.as_ref()); //, &mut self.model.as_mut_ptr());
    }

    /// add an item to the pulldown
    ///
    /// # Arguments
    /// * The item to be found, as a &MutPtr<QString>
    pub fn find_item<'a>(&mut self, item: QRef<QString>) -> Option<MutPtr<QStandardItem>> {
        unsafe {
            return Self::_find_item(item, &self.model.as_mut_ptr());
        }
    }

    /// scroll to the provided item in the list
    ///
    /// # Arguments
    /// * `item` - A Ref wrapped QString.
    /// * `select1 - a boolean indicating whether the item should be selected as well as centered
    /// in the view
    pub fn scroll_to_item<'a>(&mut self, item: QRef<QString>, select_item: bool) {
        unsafe {
            Self::_scroll_to_item(
                item,
                &mut self.view,
                &mut self.model.as_mut_ptr(),
                select_item,
            );
        }
    }

    /// Select the provided item given a Ref wrapped QModelIndex
    ///
    /// # Arguments
    /// * `item` - QModelIndex of the item we wish to select
    ///
    /// # Returns
    /// * None
    #[allow(dead_code)]
    pub fn select_item(&mut self, item: QRef<QModelIndex>) {
        unsafe {
            Self::_select_item(item, &self.view);
        }
    }

    #[allow(dead_code)]
    /// Delete selected items from the list.
    ///
    /// # Arguments
    /// None
    ///
    /// # Returns
    /// None
    pub fn delete_sel_items(&mut self) {
        unsafe {
            let selected = self.view.selection_model().selected_indexes();
            if selected.length() == 0 {
                return;
            }
            for c in 0..selected.length() {
                self.view.model().remove_row_1a(c);
            }
        }
    }

    pub fn items(&self) -> Vec<String> {
        self.items.borrow().items()
    }

    #[allow(dead_code)]
    /// Set comboboc items, replacing any extant items
    ///
    /// # Arguments
    /// * `items` - Vector of items that may be converted to a &str vie Into<&'cstr>
    pub fn set_cb_items<'c, I>(&mut self, items: Vec<I>)
    where
        I: AsRef<str>,
    {
        unsafe {
            self.remove_cb_items();
            self.add_combobox.add_item_q_string(&qs(""));
            for item in items {
                self.add_combobox.add_item_q_string(&qs(item.as_ref()));
            }
        }
    }

    #[allow(dead_code)]
    /// Remove all items from the combobox
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns None
    pub fn remove_cb_items(&mut self) {
        unsafe {
            self.add_combobox.clear();
        }
    }

    /// Given a path as a &str to a stylesheet, apply it to the components.
    ///
    /// # Arguments
    /// * `sheet` - Path to the qss stylesheet
    ///
    /// # Returns
    /// * None
    pub fn set_stylesheet(&mut self, sheet: &str) {
        load_stylesheet(sheet, self.main);
    }

    fn _find_item<'a>(
        item: QRef<QString>,
        model: &MutPtr<QStandardItemModel>,
    ) -> Option<MutPtr<QStandardItem>> {
        unsafe {
            let mut location = model.find_items_2a(item, MatchFlag::MatchCaseSensitive.into());
            if location.count() == 0 {
                return None;
            }
            let first = location.take_first();
            Some(first)
        }
    }

    fn _scroll_to_item<'a>(
        item: QRef<QString>,
        view: &mut MutPtr<QListView>,
        model: &mut MutPtr<QStandardItemModel>,
        select: bool,
    ) -> bool {
        unsafe {
            if let Some(item) = Self::_find_item(item, model) {
                let idx = item.index();
                view.scroll_to_1a(&idx);
                if select == true {
                    Self::_select_item(idx.as_ref(), &view);
                }
                return true;
            }
            false
        }
    }

    unsafe fn _select_item(item: QRef<QModelIndex>, view: &MutPtr<QListView>) {
        view.selection_model().clear();
        view.selection_model()
            .set_current_index(item, SelectionFlag::SelectCurrent.into());
    }

    #[allow(dead_code)]
    pub fn set_add_mode(&mut self) {
        unsafe {
            self.mode_toolbar
                .borrow_mut()
                .add_mode_action
                .activate(ActionEvent::Trigger);
        }
    }

    #[allow(dead_code)]
    pub fn set_find_mode(&mut self) {
        unsafe {
            self.mode_toolbar
                .borrow_mut()
                .find_mode_action
                .activate(ActionEvent::Trigger);
        }
    }
    // setup the main widget, performing configuration, adding a
    // layout, and registering ti with its parent, inserting it into
    // its parent's layout
    //
    // # Arguments
    // * `parent` - reference to the parent widget
    //
    // # Returns
    // * MutPtr to the main widget
    fn setup_main_widget(parent: &MutPtr<QWidget>) -> MutPtr<QWidget> {
        QWidget::create(&parent).add_layout(LayoutType::VBoxLayout)
    }

    // construct a model, configurng it for the listview
    //
    // # Arguments
    // * None
    //
    // # Returns
    // CppBoxed QStandardItemModel instance
    fn setup_model() -> CppBox<QStandardItemModel> {
        unsafe {
            let mut model = QStandardItemModel::new_0a();
            model.set_column_count(1);
            model
        }
    }

    // Given a name and a parent, construct a QComboBox and return it
    //
    // #Arguments
    // * `name` - Name of the combobox
    // * `parent` - mut reference to the parent widget. Will be used to fetch the layout
    //
    // # Returns
    // * A MutPtr wrapping the QComboBox
    fn setup_combobox(
        name: &str,
        mut parent: &mut MutPtr<QWidget>,
    ) -> (MutPtr<QLabel>, MutPtr<QComboBox>) {
        unsafe {
            let mut cb_widget = QFrame::create(&mut parent);
            cb_widget.add_layout(LayoutType::HBoxLayout);
            cb_widget.set_object_name(&qs(format!("{}Widget", name)));

            let mut cb_label = QLabel::from_q_string(&qs("Add Item"));
            cb_label.set_object_name(&qs("WithsCBLabel"));
            let cb_label_ptr = cb_label.as_mut_ptr();
            cb_widget.layout().add_widget(cb_label.into_ptr());

            let mut cbox = QComboBox::new_0a();
            cbox.set_editable(true);
            cbox.set_object_name(&qs("WithsComboBox"));
            let cbox_ptr = cbox.as_mut_ptr();
            cb_widget.layout().add_widget(cbox.into_ptr());

            let mut layout = cb_widget.layout().dynamic_cast_mut::<QHBoxLayout>();
            if layout.is_null() {
                log::error!("unable to cast layout to QHBoxLayout");
                return (cb_label_ptr, cbox_ptr);
            }
            layout.set_stretch(1, 1);

            (cb_label_ptr, cbox_ptr)
        }
    }

    // set up the ListView, configuring drag and drop, registering
    // the model, and adding it into the supplied layout
    //
    // # Arguments
    // * `model` - the instance of the QStandardItemModel, configured
    // * `layout` - The parent layout
    //
    // # Returns
    // * MutPtr wrapped QListView instance
    fn setup_listview(
        model: MutPtr<QStandardItemModel>,
        layout: &mut MutPtr<QLayout>,
    ) -> MutPtr<QListView> {
        unsafe {
            let mut qlv = QListView::new_0a();
            qlv.set_object_name(&qs("WithsListView"));
            qlv.set_model(model);
            qlv.set_drag_enabled(true);
            qlv.set_selection_mode(SelectionMode::ExtendedSelection);
            qlv.set_drag_drop_overwrite_mode(false);
            qlv.set_drag_drop_mode(DragDropMode::InternalMove);
            let qlv_ptr = qlv.as_mut_ptr();
            layout.add_widget(qlv.into_ptr());

            qlv_ptr
        }
    }

    unsafe fn setup_button(name: &str, layout: &mut MutPtr<QLayout>) -> MutPtr<QPushButton> {
        let mut button = QPushButton::from_q_string(&qs(name));
        let button_ptr = button.as_mut_ptr();
        layout.add_widget(button.into_ptr());
        button_ptr
    }
}
