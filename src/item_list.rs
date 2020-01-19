//use super::list_items::ListItems;
use super::utility::qs;
//use crate::toolbar::ItemListModeToolbar;
pub use crate::traits::*;
//use crate::utility::load_stylesheet;
use log;
use qt_core::{q_item_selection_model::SelectionFlag, Key, MatchFlag, QModelIndex, QString, Slot};
use qt_gui::{q_key_sequence::StandardKey, QKeySequence, QStandardItem, QStandardItemModel};
//use qt_widgets::cpp_core::Ref as QRef;
use crate::inner_item_list::InnerItemList;
use qt_widgets::{
    cpp_core::MutPtr,
    cpp_core::Ref as QRef,
    // q_abstract_item_view::DragDropMode,
    // q_abstract_item_view::SelectionMode,
    // q_action::ActionEvent,
    //QComboBox, QFrame, QHBoxLayout, QLabel, QLayout,
    QListView,
    //QPushButton,
    QShortcut,
    QWidget,
};
pub use rustqt_utils::{as_mut_ref, as_ref, enclose, enclose_all};
//use std::cell::RefCell;
use std::rc::Rc;

//
// ITEMLIST
//
/// The ItemList provides a litview with a toolbar allowing you
/// to switch between reordering, deleting, and adding members.
/// It stores the main components that are interesting to
/// its clients, including the toolbar, the model, the view,
/// the actual items backing data, and various slots
pub struct ItemList<'l> {
    pub inner: Rc<InnerItemList>,
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
    pub fn new(parent: MutPtr<QWidget>) -> ItemList<'l> {
        unsafe {
            let inner = Rc::new(InnerItemList::new(parent));

            // shortcuts
            let enter_key_seq = QKeySequence::from_int(Key::KeyReturn.to_int());
            let enter_shortcut = QShortcut::new_2a(enter_key_seq.as_ref(), inner.main());

            let key_seq = QKeySequence::from_int(Key::KeyBackspace.to_int());
            let delete_shortcut = QShortcut::new_2a(key_seq.as_ref(), inner.main());

            let cut_key_seq = QKeySequence::from_standard_key(StandardKey::Cut);
            let cut_shortcut = QShortcut::new_2a(cut_key_seq.as_ref(), inner.main());

            let inner_view = inner.view();
            let rm_slot = Slot::new(enclose_all! { () (mut inner_view) move || {
                let selected = inner_view.selection_model().selected_indexes();
                if selected.length() == 0 {
                    return;
                }
                // we need to sort the indexes first. Otherwise, depending upon selection order, we
                // may not
                let mut indexes = (0..selected.size()).into_iter().map(|x| selected.at(x).row()).collect::<Vec<_>>();
                indexes.sort();
                indexes.iter().rev().for_each(|c| {inner_view.model().remove_row_1a(*c); });
            }});

            // let mode_toolbar = inner.toolbar();
            let cbox_ptr = inner.add_combobox();
            let listview_ptr = inner.view();
            let model_ptr = inner.model();

            let enter_sc = Slot::new(
                enclose_all! { (inner) (mut cbox_ptr, mut listview_ptr, mut model_ptr) move || {
                    let text = cbox_ptr.current_text();
                    if inner.is_find_active() {
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

                    inner.add_item_to(text.to_std_string().as_str());
                    cbox_ptr.clear_edit_text();
                    listview_ptr.scroll_to_bottom();

                }},
            );
            let cblabel = inner.add_label();
            let f = Self {
                inner,
                enter_shortcut: enter_shortcut.into_ptr(),
                delete_shortcut: delete_shortcut.into_ptr(),
                cut_shortcut: cut_shortcut.into_ptr(),
                rm: rm_slot,

                find_mode: Slot::new(
                    as_mut_ref! { (cblabel) enclose_all! { () ( mut cbox_ptr) move || {
                        cbox_ptr.set_enabled(true);
                        if let Some(mut cblabel) = cblabel {cblabel.set_text(&qs("Find Item"))};
                    }}},
                ),

                add_mode: Slot::new(
                    as_mut_ref! {(cblabel) enclose_all! { () ( mut cbox_ptr) move || {
                        cbox_ptr.set_enabled(true);
                        if let Some(mut cblabel) = cblabel {cblabel.set_text(&qs("Add Item"))};

                    }}},
                ),

                enter_sc,
            };

            f.inner()
                .find_mode_action()
                .triggered()
                .connect(&f.find_mode);

            f.inner().add_mode_action().triggered().connect(&f.add_mode);

            f.enter_shortcut.activated().connect(&f.enter_sc);

            f.delete_shortcut.activated().connect(&f.rm);

            f.cut_shortcut.activated().connect(&f.rm);

            f
        }
    }

    /// Retrieve an RC wrapped InnerItemList instance
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * Rc of the InnerItemList instance
    pub fn inner(&self) -> Rc<InnerItemList> {
        self.inner.clone()
    }

    /// Retrieve a mutable pointer to the component's top QWidget. That is
    /// the widget contained within that is the parent of the other internal
    /// widgets.
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * Mutable Pointer to the main QWidget
    pub fn main(&self) -> MutPtr<QWidget> {
        self.inner().main()
    }

    #[allow(dead_code)]
    /// Clear the listview and its backng model
    ///
    /// # Arguments
    /// None
    ///
    /// # Returns
    /// None
    pub fn clear(&self) {
        self.inner().clear();
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
    pub fn set_items<I>(&self, items: Vec<I>)
    where
        I: AsRef<str>,
    {
        self.clear();
        let inner = self.inner();
        for item in items {
            inner.add_item(item.as_ref());
        }
    }

    /// Retrieve the model for the component
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * MutPtr wrapping QStandardItemModel
    pub fn model(&self) -> MutPtr<QStandardItemModel> {
        self.inner().model()
    }

    /// Retrieve the primary list view.
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * MutPtr to QListView
    pub fn view(&self) -> MutPtr<QListView> {
        self.inner().view()
    }

    /// add an item to the pulldown
    ///
    /// # Arguments
    /// * The item to be added, as a &str or String
    ///
    /// # Returns
    /// * None
    pub fn add_item<I>(&self, item: I)
    where
        I: AsRef<str>,
    {
        self.inner().add_item_to(item.as_ref()); //, &mut self.model.as_mut_ptr());
    }

    /// add an item to the pulldown
    ///
    /// # Arguments
    /// * The item to be found, as a &MutPtr<QString>
    pub fn find_item<'a>(&self, item: QRef<QString>) -> Option<MutPtr<QStandardItem>> {
        return Self::_find_item(item, &self.model());
    }

    /// scroll to the provided item in the list
    ///
    /// # Arguments
    /// * `item` - A Ref wrapped QString.
    /// * `select1 - a boolean indicating whether the item should be selected as well as centered
    /// in the view
    ///
    /// # Returns
    /// * None
    pub fn scroll_to_item<'a>(&self, item: QRef<QString>, select_item: bool) {
        Self::_scroll_to_item(item, &mut self.view(), &mut self.model(), select_item);
    }

    /// Select the provided item given a Ref wrapped QModelIndex
    ///
    /// # Arguments
    /// * `item` - QModelIndex of the item we wish to select
    ///
    /// # Returns
    /// * None
    #[allow(dead_code)]
    pub fn select_item(&self, item: QRef<QModelIndex>) {
        unsafe {
            Self::_select_item(item, &self.view());
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
    pub fn delete_sel_items(&self) {
        unsafe {
            let selected = self.view().selection_model().selected_indexes();
            if selected.length() == 0 {
                return;
            }
            let mut view_model = self.view().model();
            for c in 0..selected.length() {
                view_model.remove_row_1a(c);
            }
        }
    }

    /// Get the items as a vector of Strings.
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * Vector of String
    pub fn items(&self) -> Vec<String> {
        self.inner().items()
    }

    #[allow(dead_code)]
    /// Set comboboc items, replacing any extant items
    ///
    /// # Arguments
    /// * `items` - Vector of items
    ///
    /// # Returns
    /// * None
    pub fn set_cb_items<'c, I>(&self, items: Vec<I>)
    where
        I: AsRef<str>,
    {
        self.inner().set_cb_items(items);
    }

    #[allow(dead_code)]
    /// Remove all items from the combobox
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns None
    pub fn remove_cb_items(&self) {
        self.inner().remove_cb_items();
    }

    /// Change the max number of items displayed in the combobox's dropdown
    /// list
    ///
    /// # Arguments
    /// * `max` - Maximum number of visible items in the comobobox's dropdown
    ///
    /// # Returns
    /// * None
    pub fn set_cb_max_visible_items(&self, max: i32) {
        self.inner().set_cb_max_visible_items(max);
    }

    /// Given a path as a &str to a stylesheet, apply it to the components.
    ///
    /// # Arguments
    /// * `sheet` - Path to the qss stylesheet
    ///
    /// # Returns
    /// * None
    pub fn set_stylesheet(&self, sheet: &str) {
        self.inner().set_stylesheet(sheet);
    }

    /// Set the component to add mode
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * None
    pub fn set_add_mode(&self) {
        self.inner().set_add_mode();
    }

    #[allow(dead_code)]
    /// Set the component to find mode
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * None
    pub fn set_find_mode(&self) {
        self.inner().set_find_mode();
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
}
