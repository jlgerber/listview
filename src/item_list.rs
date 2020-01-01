use super::list_items::ListItems;
use qt_gui::QStandardItemModel;
use qt_widgets::q_abstract_item_view::DragDropMode;
use qt_widgets::{
    cpp_core::{CppBox, MutPtr},
    QLayout, QListView, QVBoxLayout, QWidget,
};

pub struct ItemList {
    _main: MutPtr<QWidget>,
    model: CppBox<QStandardItemModel>,
    view: MutPtr<QListView>,
    items: ListItems,
}

impl ItemList {
    pub fn new(parent: MutPtr<QWidget>) -> ItemList {
        unsafe {
            let main_ptr = Self::setup_main_widget(&parent);
            let listitems = ListItems::new();
            let mut model = Self::setup_model();
            let listview_ptr = Self::setup_listview(model.as_mut_ptr(), &mut main_ptr.layout());
            let f = Self {
                _main: main_ptr,
                model,
                view: listview_ptr,
                items: listitems,
            };
            f
        }
    }
    fn setup_main_widget(parent: &MutPtr<QWidget>) -> MutPtr<QWidget> {
        unsafe {
            let mut main = QWidget::new_0a();
            let mut main_ptr = main.as_mut_ptr();

            let mut parent_layout = parent.layout();
            parent_layout.add_widget(main.into_ptr());

            let layout = QVBoxLayout::new_0a();
            main_ptr.set_layout(layout.into_ptr());
            main_ptr
        }
    }
    fn setup_model() -> CppBox<QStandardItemModel> {
        unsafe {
            let mut model = QStandardItemModel::new_0a();
            model.set_column_count(1);
            model
        }
    }
    fn setup_listview(
        model: MutPtr<QStandardItemModel>,
        layout: &mut MutPtr<QLayout>,
    ) -> MutPtr<QListView> {
        unsafe {
            let mut qlv = QListView::new_0a();
            qlv.set_model(model);
            qlv.set_drag_enabled(true);
            qlv.set_drag_drop_overwrite_mode(false);
            qlv.set_drag_drop_mode(DragDropMode::InternalMove);
            let qlv_ptr = qlv.as_mut_ptr();
            layout.add_widget(qlv.into_ptr());
            qlv_ptr
        }
    }

    /// add an item to the pulldown
    pub fn add_item(&mut self, item: &str) {
        unsafe {
            self.items.add_item_to(item, &mut self.model.as_mut_ptr());
        }
    }

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
}
