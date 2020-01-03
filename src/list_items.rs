use super::utility::qs;
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::cpp_core::MutPtr;

pub struct ListItems {
    items: Vec<MutPtr<QStandardItem>>,
}

impl ListItems {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn clear(&mut self) {
        self.items.clear();
    }
    pub fn add_item_to(&mut self, item: &str, model: &mut MutPtr<QStandardItemModel>) {
        unsafe {
            let mut si = QStandardItem::new();
            si.set_text(&qs(item));
            si.set_drop_enabled(false);
            model.append_row_q_standard_item(si.as_mut_ptr());
            self.items.push(si.into_ptr());
        }
    }
}
