use super::utility::qs;
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::cpp_core::MutPtr;

/// Wrapper around a Vector of QStandardItems which provides a bit of an api.
pub struct ListItems {
    items: Vec<MutPtr<QStandardItem>>,
    model: MutPtr<QStandardItemModel>,
}

impl ListItems {
    /// New up an empty ListItems instance
    pub fn new(model: MutPtr<QStandardItemModel>) -> Self {
        Self {
            items: Vec::new(),
            model,
        }
    }
    /// Clear the items from the list
    ///
    /// # Arguments
    /// ^ None
    ///
    /// # Returns
    /// None
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Add an item to self.
    ///
    /// # Arguments
    /// * iteem - The name of the item
    ///
    /// # Returns
    /// * Noen
    pub fn add_item_to(&mut self, item: &str) {
        unsafe {
            let mut si = QStandardItem::new();
            si.set_text(&qs(item));
            si.set_drop_enabled(false);
            self.model.append_row_q_standard_item(si.as_mut_ptr());
            self.items.push(si.into_ptr());
        }
    }
}
