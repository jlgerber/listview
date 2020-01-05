//!
//! NO LONGER IN USE. WILL BE REMOVING
use crate::item_list::*;
use crate::utility::load_stylesheet;
use crate::utility::qs;
use qt_core::Key;
use qt_gui::{q_key_sequence::StandardKey, QKeySequence};
use qt_widgets::{
    cpp_core::{CppBox, MutPtr, MutRef},
    QLayout, QPushButton, QShortcut, QVBoxLayout, QWidget,
};
use std::cell::RefCell;
use std::rc::Rc;

// makes it simpler to deal with the need to clone. Saw this here:
// https://github.com/rust-webplatwith_list/rust-todomvc/blob/master/src/main.rs#L142
#[allow(unused_macros)]
macro_rules! enclose {
    ( ($(  $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[allow(unused_macros)]
macro_rules! enclose_mut {
    ( ($( mut $x:ident ),*) $y:expr ) => {
        {
            $(let mut $x = $x.clone();)*
            $y
        }
    };
}

/// clone both immutable and mutable vars. Useful for
/// qt, which has a lot more mutable
/// use like so:
/// ```ignore
/// Slot::,new(enclose_all!{ (foo, bar) (mut bla) move || {}}),
/// ```
#[allow(unused_macros)]
macro_rules! enclose_all {
    ( ($(  $x:ident ),*) ($( mut $mx:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $(let mut $mx = $mx.clone();)*
            $y
        }
    };
}
// Decided that the following is not needed. My initial intention was to allow for
// both owned and non-owned alternative for the struct. However, upon reflection, it seems
// more reasonable to simply pass in an owning QWidget.
//pub enum QtState<T> {
//    Owned(CppBox<T>),
//    Held(MutRef<T>)
//}
//
//impl<T> enum QState<T> {
//    pub fn from_owned(item: CppBox<T>) -> Self {
//        Self::Owned(item)
//    }
//    pub fn from_held(item: MutRef<T>) -> Self {
//        Self::Held(item)
//    }
//
//    pub fn to_held(self) -> Self {
//        match self {
//            Self::Owned()
//        }
//    }
//}
/// The WithList provides an editable, reorderable
/// list of items that may be multi-selected and deleted.
/// There are three major visual components:
/// * A set of radio mode buttons
/// * A combobox used to create and find
/// * A listview which displays items
///
/// really, the only thing that WithList provides is the save button
/// and a couple of shortcuts. We should eliminate it
pub struct WithList<'a> {
    /// A pointer to the internal parent widget in the list
    pub main: MutPtr<QWidget>,
    /// The ItemList instance
    pub item_list: Rc<RefCell<ItemList<'a>>>,
    pub delete_shortcut: MutPtr<QShortcut>,
    pub cut_shortcut: MutPtr<QShortcut>,
}

impl<'a> WithList<'a> {
    pub fn new(parent: &mut MutRef<QWidget>) -> Self {
        unsafe {
            let mut main = Self::setup_main();
            let main_ptr = main.as_mut_ptr();
            parent.layout().add_widget(main.into_ptr());
            let item_list = Rc::new(RefCell::new(ItemList::new(main_ptr)));
            //buttons
            let _save_button = Self::setup_button("Save", &mut main_ptr.layout());
            // shortcuts
            let key_seq = QKeySequence::from_int(Key::KeyBackspace.to_int());
            let delete_shortcut = QShortcut::new_2a(key_seq.as_ref(), main_ptr);

            let cut_key_seq = QKeySequence::from_standard_key(StandardKey::Cut);
            let cut_shortcut = QShortcut::new_2a(cut_key_seq.as_ref(), main_ptr);

            let f = WithList {
                main: main_ptr,
                item_list: item_list.clone(),
                delete_shortcut: delete_shortcut.into_ptr(),
                cut_shortcut: cut_shortcut.into_ptr(),
            };
            f.delete_shortcut
                .activated()
                .connect(&f.item_list.borrow_mut().rm);

            f.cut_shortcut
                .activated()
                .connect(&f.item_list.borrow_mut().rm);

            f
        }
    }

    pub fn show(&mut self) {
        unsafe {
            self.main.show();
        }
    }

    pub fn main(&mut self) -> MutPtr<QWidget> {
        self.main
    }

    pub fn set_stylesheet(&mut self, sheet: &str) {
        load_stylesheet(sheet, self.main());
    }

    pub fn add_items<'i: 'a, I>(&self, items: Vec<I>)
    where
        I: Into<&'i str>,
    {
        self.item_list.borrow_mut().set_cb_items(items);
    }

    pub fn set_add_mode(&self) {
        self.item_list.borrow_mut().set_add_mode();
    }

    pub fn set_find_mode(&self) {
        self.item_list.borrow_mut().set_find_mode();
    }

    unsafe fn setup_main() -> CppBox<QWidget> {
        let mut main = QWidget::new_0a();
        let layout = QVBoxLayout::new_0a();
        main.set_layout(layout.into_ptr());
        main
    }

    unsafe fn setup_button(name: &str, layout: &mut MutPtr<QLayout>) -> MutPtr<QPushButton> {
        let mut button = QPushButton::from_q_string(&qs(name));
        let button_ptr = button.as_mut_ptr();
        layout.add_widget(button.into_ptr());
        button_ptr
    }
}
