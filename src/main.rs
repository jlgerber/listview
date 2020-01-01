use qt_core::{QListOfQModelIndex, QString, QStringList, QStringListModel};
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::q_abstract_item_view::DragDropMode;
use qt_widgets::{
    cpp_core::{CppBox, MutPtr, Ptr, Ref},
    QApplication, QListView, QVBoxLayout, QWidget, SlotOfQListOfQModelIndex,
};

// Given an input of &str or String, return a boxed QString
pub fn qs<S: AsRef<str>>(input: S) -> CppBox<QString> {
    QString::from_std_str(input.as_ref())
}

struct ListItems {
    items: Vec<MutPtr<QStandardItem>>,
}
impl ListItems {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add_item_to(&mut self, item: &str, model: &mut MutPtr<QStandardItemModel>) {
        unsafe {
            let mut si = QStandardItem::new();
            si.set_text(&qs(item));
            si.set_drop_enabled(false);
            model.append_row_q_standard_item(si.as_mut_ptr());
            self.items.push(si.into_ptr());
        }
    }
}

struct Form<'a> {
    _main: CppBox<QWidget>,
    _model: CppBox<QStandardItemModel>,
    _items: ListItems,
    index_moved: SlotOfQListOfQModelIndex<'a>,
}

impl<'a> Form<'a> {
    fn new() -> Self {
        unsafe {
            let mut main = QWidget::new_0a();
            let mut layout = QVBoxLayout::new_0a();
            let mut layout_ptr = layout.as_mut_ptr();
            main.set_layout(layout.into_ptr());
            let mut listitems = ListItems::new();

            let mut slm = QStandardItemModel::new_0a();
            let mut slm_ptr = slm.as_mut_ptr();

            slm.set_column_count(1);
            listitems.add_item_to("One", &mut slm_ptr);
            listitems.add_item_to("Two", &mut slm_ptr);
            listitems.add_item_to("Three", &mut slm_ptr);

            let mut qlv = QListView::new_0a();
            let mut qlv_ptr = qlv.as_mut_ptr();
            qlv.set_model(slm.as_mut_ptr());
            qlv.set_drag_enabled(true);
            qlv.set_drag_drop_overwrite_mode(false);
            qlv.set_drag_drop_mode(DragDropMode::InternalMove);
            layout_ptr.add_widget(qlv.into_ptr());
            main.show();
            let f = Form {
                _main: main,
                _model: slm,
                _items: listitems,
                index_moved: SlotOfQListOfQModelIndex::new(
                    move |indexlist: Ref<QListOfQModelIndex>| {
                        println!("ohboy");
                    },
                ),
            };
            qlv_ptr.indexes_moved().connect(&f.index_moved);
            f
        }
    }
}

fn main() {
    QApplication::init(|_app| unsafe {
        let mut _form = Form::new();
        QApplication::exec()
    });
}
