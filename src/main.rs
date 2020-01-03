use qt_core::{Key, QResource};
use qt_widgets::{
    cpp_core::{CppBox, MutPtr},
    QApplication, QLayout, QPushButton, QShortcut, QVBoxLayout, QWidget,
};
use std::cell::RefCell;
use std::rc::Rc;
mod item_list;
use item_list::*;
pub mod utility;
use utility::load_stylesheet;
use utility::qs;
pub mod list_items;
use qt_gui::{q_key_sequence::StandardKey, QKeySequence};

// makes it simpler to deal with the need to clone. Saw this here:
// https://github.com/rust-webplatform/rust-todomvc/blob/master/src/main.rs#L142
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

struct WithList<'a> {
    main: CppBox<QWidget>,
    item_list: Rc<RefCell<ItemList<'a>>>,
    delete_shortcut: MutPtr<QShortcut>,
    cut_shortcut: MutPtr<QShortcut>,
}

impl<'a> WithList<'a> {
    fn new() -> Self {
        unsafe {
            let mut main = Self::setup_main();
            let main_ptr = main.as_mut_ptr();
            let item_list = Rc::new(RefCell::new(ItemList::new(main_ptr)));
            //buttons
            let _save_button = Self::setup_button("Save", &mut main.layout());
            // shortcuts
            let key_seq = QKeySequence::from_int(Key::KeyBackspace.to_int());
            let delete_shortcut = QShortcut::new_2a(key_seq.as_ref(), main_ptr);

            let cut_key_seq = QKeySequence::from_standard_key(StandardKey::Cut);
            let cut_shortcut = QShortcut::new_2a(cut_key_seq.as_ref(), main_ptr);

            let f = WithList {
                main: main,
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
        unsafe { self.main.as_mut_ptr() }
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

fn main() {
    QApplication::init(|_app| unsafe {
        let _result = QResource::register_resource_q_string(&qs("/Users/jgerber/bin/listitem.rcc"));

        let mut form = WithList::new();
        form.set_stylesheet("/Users/jgerber/bin/listitem.qss");
        form.show();
        form.item_list
            .borrow_mut()
            .set_items(vec!["Foo", "bar", "bla"]);
        form.add_items(vec![
            "Foo",
            "Bar",
            "Bla",
            "Fred",
            "Barney",
            "Ralph",
            "Cedrick",
            "animpublish",
            "animrender",
        ]);
        QApplication::exec()
    });
}
