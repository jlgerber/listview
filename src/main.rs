use qt_core::Slot;
use qt_widgets::{cpp_core::CppBox, QApplication, QPushButton, QVBoxLayout, QWidget};
use std::cell::RefCell;
use std::rc::Rc;
mod item_list;
use item_list::*;
pub mod utility;
use utility::load_stylesheet;
use utility::qs;
pub mod list_items;

struct Form<'a> {
    _main: CppBox<QWidget>,
    _item_list: Rc<RefCell<ItemList>>,
    rm: Slot<'a>,
    add: Slot<'a>,
}

impl<'a> Form<'a> {
    fn new() -> Self {
        unsafe {
            let mut main = QWidget::new_0a();
            let main_ptr = main.as_mut_ptr();
            load_stylesheet(
                "/Users/jgerber/src/rust/examples/qt/listitem/stylesheet.qss",
                main.as_mut_ptr(),
            );
            let mut layout = QVBoxLayout::new_0a();
            let mut layout_ptr = layout.as_mut_ptr();
            main.set_layout(layout.into_ptr());
            let item_list = Rc::new(RefCell::new(ItemList::new(main_ptr)));
            let item_list_cpy = item_list.clone();
            let item_list_cpy2 = item_list.clone();
            // add button
            let mut rm_button = QPushButton::from_q_string(&qs("Remove"));
            let rm_button_ptr = rm_button.as_mut_ptr();
            layout_ptr.add_widget(rm_button.into_ptr());

            let mut add_button = QPushButton::from_q_string(&qs("Add"));
            let add_button_ptr = add_button.as_mut_ptr();
            layout_ptr.add_widget(add_button.into_ptr());

            main.show();
            let f = Form {
                _main: main,
                _item_list: item_list,
                rm: Slot::new(move || {
                    item_list_cpy.borrow_mut().delete_sel_items();
                }),
                add: Slot::new(move || {
                    item_list_cpy2.borrow_mut().add_item("New Item");
                }),
            };
            rm_button_ptr.clicked().connect(&f.rm);
            add_button_ptr.clicked().connect(&f.add);

            f
        }
    }
    fn setup_main() -> CppBox<QWidget> {
        unsafe {
            let mut main = QWidget::new_0a();
            let mut layout = QVBoxLayout::new_0a();
            let layout_ptr = layout.as_mut_ptr();
            main.set_layout(layout.into_ptr());
            main
        }
    }
}

fn main() {
    QApplication::init(|_app| unsafe {
        let mut _form = Form::new();
        QApplication::exec()
    });
}
