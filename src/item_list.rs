use super::list_items::ListItems;
use super::utility::qs;
use qt_core::Slot;
use qt_gui::{q_key_sequence::StandardKey, QKeySequence, QStandardItemModel};
use qt_widgets::q_abstract_item_view::DragDropMode;
use qt_widgets::{
    cpp_core::{CppBox, MutPtr, MutRef},
    q_abstract_item_view::SelectionMode,
    q_size_policy::Policy,
    QAction, QActionGroup, QHBoxLayout, QLabel, QLayout, QListView, QShortcut, QSizePolicy,
    QToolBar, QToolButton, QVBoxLayout, QWidget,
};
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! enclose {
    ( ($(  $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

/// New up a widget,
pub unsafe trait NewWidget<P, R> {
    fn create(parent: &MutPtr<P>) -> MutPtr<R>;
}

unsafe impl NewWidget<QWidget, QWidget> for QWidget {
    fn create(parent: &MutPtr<QWidget>) -> MutPtr<QWidget> {
        unsafe {
            let mut main = QWidget::new_0a();
            let main_ptr = main.as_mut_ptr();
            let mut parent_ptr = parent.layout();
            assert!(!parent_ptr.is_null());
            parent_ptr.add_widget(main.into_ptr());
            main_ptr
        }
    }
}

#[allow(dead_code)]
pub enum LayoutType {
    VBoxLayout,
    HBoxLayout,
}

pub unsafe trait AddLayout<R> {
    type Layout;
    fn add_layout(&mut self, layout: Self::Layout) -> MutPtr<R>;
}

fn add_layout_to_widget(widget: &mut MutPtr<QWidget>, layout: LayoutType) {
    unsafe {
        match layout {
            LayoutType::VBoxLayout => {
                let mut layout = QVBoxLayout::new_0a();
                layout.set_margin(0);
                layout.set_contents_margins_4a(0, 0, 0, 0);
                layout.set_spacing(0);
                widget.set_layout(layout.into_ptr());
            }
            LayoutType::HBoxLayout => {
                let mut layout = QHBoxLayout::new_0a();
                layout.set_margin(0);
                layout.set_contents_margins_4a(0, 0, 0, 0);
                layout.set_spacing(0);
                widget.set_layout(layout.into_ptr());
            }
        }
    }

    unsafe impl AddLayout<QWidget> for MutPtr<QWidget> {
        type Layout = LayoutType;

        fn add_layout(&mut self, layout: LayoutType) -> MutPtr<QWidget> {
            unsafe {
                add_layout_to_widget(self, layout);
                self.as_mut_ref().unwrap().as_mut_ptr()
            }
        }
    }
}

pub struct ItemListModeToolbar<'a> {
    pub toolbar: MutPtr<QToolBar>,
    pub action_group: MutPtr<QActionGroup>,
    pub save_action: MutPtr<QAction>,
    pub reorder_mode_action: MutPtr<QAction>,
    pub rm_mode_action: MutPtr<QAction>,
    pub add_mode_action: MutPtr<QAction>,
    pub edit: Slot<'a>,
}

impl<'a> ItemListModeToolbar<'a> {
    unsafe fn create_toolbar(name: &str) -> CppBox<QToolBar> {
        let mut toolbar = QToolBar::from_q_string(&qs(name));
        toolbar.set_floatable(false);
        toolbar.set_movable(false);
        toolbar.add_widget(QLabel::from_q_string(&qs("Mode:")).into_ptr());
        toolbar
    }
    unsafe fn create_spacer() -> CppBox<QWidget> {
        let mut spacer = QWidget::new_0a();
        let sp = QSizePolicy::new_2a(Policy::Expanding, Policy::Fixed);
        spacer.set_size_policy_1a(sp.as_ref());
        spacer
    }
    //
    #[allow(dead_code)]
    unsafe fn create_mode_action(
        name: &str,
        action_grp_ptr: MutPtr<QActionGroup>,
        toolbar: &mut MutPtr<QToolBar>,
        checked: bool,
    ) -> CppBox<QAction> {
        let mut mode_action = QAction::from_q_string_q_object(&qs(name), action_grp_ptr);
        mode_action.set_checkable(true);
        mode_action.set_checked(checked);
        toolbar.add_action(mode_action.as_mut_ptr());
        let mut button: MutPtr<QToolButton> = toolbar
            .widget_for_action(mode_action.as_mut_ptr())
            .dynamic_cast_mut();
        button.set_object_name(&qs("WithpackagesToolbarButton"));

        mode_action
    }

    pub fn new(parent: &mut MutPtr<QWidget>) -> Self {
        unsafe {
            let mut toolbar = Self::create_toolbar("WithPackage Toolbar");
            let mut action_group = QActionGroup::new(toolbar.as_mut_ptr());
            let action_group_ptr = action_group.as_mut_ptr();
            // add spacer widget
            let mut spacer = Self::create_spacer();
            //
            let mut reorder_mode_action =
                QAction::from_q_string_q_object(&qs("Reorder"), action_group_ptr);
            reorder_mode_action.set_checkable(true);
            reorder_mode_action.set_checked(true);
            toolbar.add_action(reorder_mode_action.as_mut_ptr());
            //
            let mut rm_mode_action =
                QAction::from_q_string_q_object(&qs("Remove"), action_group_ptr);
            rm_mode_action.set_checkable(true);
            toolbar.add_action(rm_mode_action.as_mut_ptr());
            //
            let mut add_mode_action = QAction::from_q_string_q_object(&qs("Add"), action_group_ptr);
            toolbar.add_action(add_mode_action.as_mut_ptr());
            add_mode_action.set_checkable(true);
            toolbar.add_widget(spacer.into_ptr());
            let save_action = toolbar.add_action_1a(&qs("Save"));
            // configure buttons
            let mut save_button: MutPtr<QToolButton> =
                toolbar.widget_for_action(save_action).dynamic_cast_mut();
            save_button.set_object_name(&qs("WithpackagesToolbarButton"));
            //
            let mut reorder_button: MutPtr<QToolButton> = toolbar
                .widget_for_action(reorder_mode_action.as_mut_ptr())
                .dynamic_cast_mut();
            //let  reorder_button_ref = reorder_button.as_mut_ref().unwrap();
            reorder_button.set_object_name(&qs("WithpackagesToolbarButton"));
            //
            let mut rm_button: MutPtr<QToolButton> = toolbar
                .widget_for_action(rm_mode_action.as_mut_ptr())
                .dynamic_cast_mut();
            let rm_button_ref = rm_button.as_mut_ref().unwrap();
            rm_button.set_object_name(&qs("WithpackagesToolbarButton"));
            //
            let mut add_button: MutPtr<QToolButton> = toolbar
                .widget_for_action(add_mode_action.as_mut_ptr())
                .dynamic_cast_mut();
            //let  add_button_ref = add_button.as_mut_ref().unwrap();
            add_button.set_object_name(&qs("WithpackagesToolbarButton"));
            //
            let toolbar_ptr = toolbar.as_mut_ptr();
            parent.layout().add_widget(toolbar.into_ptr());
            let edit = Slot::new(move || if rm_button_ref.is_enabled() {});
            let tb = Self {
                toolbar: toolbar_ptr,
                action_group: action_group.into_ptr(),
                save_action: save_action,
                reorder_mode_action: reorder_mode_action.into_ptr(),
                rm_mode_action: rm_mode_action.into_ptr(),
                add_mode_action: add_mode_action.into_ptr(),
                edit,
            };
            tb
        }
    }

    ///
    pub fn is_remove_active(&self) -> bool {
        unsafe { self.rm_mode_action.is_checked() }
    }
    ///
    pub fn is_add_active(&self) -> bool {
        unsafe { self.add_mode_action.is_checked() }
    }
    ///
    pub fn is_reorder_active(&self) -> bool {
        unsafe { self.reorder_mode_action.is_checked() }
    }
}
/*
pub struct ItemListToolbar<'a> {
    pub edit_action: MutPtr<QAction>,
    pub rm_action: MutPtr<QAction>,
    pub save_action: MutPtr<QAction>,
    pub edit: Slot<'a>,
}

impl<'a> ItemListToolbar<'a> {
    pub fn new(parent: &mut MutPtr<QWidget>) -> Self {
        unsafe {
            let mut toolbar = QToolBar::from_q_string(&qs("WithPackage Toolbar"));
            toolbar.set_floatable(false);
            toolbar.set_movable(false);
            // add spacer widget
            let mut spacer = QWidget::new_0a();
            let sp = QSizePolicy::new_2a(Policy::Expanding, Policy::Fixed);
            spacer.set_size_policy_1a(sp.as_ref());
            toolbar.add_widget(spacer.into_ptr());
            // add actions
            let edit_action = toolbar.add_action_1a(&qs("Edit"));
            let rm_action = toolbar.add_action_1a(&qs("Remove"));
            let save_action = toolbar.add_action_1a(&qs("Save"));
            // configure buttons
            let mut edit_button: MutPtr<QToolButton> =
                toolbar.widget_for_action(edit_action).dynamic_cast_mut();
            edit_button.set_object_name(&qs("WithpackagesToolbarButton"));
            let mut edit_button_ref = edit_button.as_mut_ref().unwrap();
            //
            let mut rm_button: MutPtr<QToolButton> =
                toolbar.widget_for_action(rm_action).dynamic_cast_mut();
            let mut rm_button_ref = rm_button.as_mut_ref().unwrap();
            rm_button_ref.set_enabled(false);
            rm_button.set_object_name(&qs("WithpackagesToolbarButton"));
            //
            let mut save_button: MutPtr<QToolButton> =
                toolbar.widget_for_action(save_action).dynamic_cast_mut();
            let mut save_button_ref = save_button.as_mut_ref().unwrap();
            save_button_ref.set_enabled(false);
            save_button.set_object_name(&qs("WithpackagesToolbarButton"));
            //
            parent.layout().add_widget(toolbar.into_ptr());
            let edit = Slot::new(move || {
                println!("clicked");
                let enabled = rm_button_ref.is_enabled();
                rm_button_ref.set_enabled(!enabled);
                save_button_ref.set_enabled(!enabled);
            });
            let tb = Self {
                edit_action,
                rm_action,
                save_action,
                edit,
            };
            edit_button.clicked().connect(&tb.edit);
            tb
        }
    }
}
*/
pub struct ItemList<'l> {
    pub _main: MutPtr<QWidget>,
    pub mode_toolbar: Rc<RefCell<ItemListModeToolbar<'l>>>,
    //pub toolbar: ItemListToolbar<'l>,
    pub model: CppBox<QStandardItemModel>,
    pub view: MutPtr<QListView>,
    pub items: Rc<RefCell<ListItems>>,
    pub delete_shortcut: MutPtr<QShortcut>,
    pub rm: Slot<'l>,
}

impl<'l> ItemList<'l> {
    /// new up an ItemList given a parent
    pub fn new(parent: MutPtr<QWidget>) -> ItemList<'l> {
        unsafe {
            let mut main_ptr = Self::setup_main_widget(&parent);
            let listitems = Rc::new(RefCell::new(ListItems::new()));
            let mut model = Self::setup_model();
            let mode_toolbar = Rc::new(RefCell::new(ItemListModeToolbar::new(&mut main_ptr)));
            let listview_ptr = Self::setup_listview(model.as_mut_ptr(), &mut main_ptr.layout());
            let key_seq = QKeySequence::from_standard_key(StandardKey::Delete);
            let delete_shortcut = QShortcut::new_2a(key_seq.as_ref(), listview_ptr);
            //let toolbar = ItemListToolbar::new(&mut main_ptr);

            let rm_slot = Slot::new(enclose! { (listview_ptr, mode_toolbar) move || {
                println!("slot called");
                if !mode_toolbar.borrow().is_remove_active() {
                    return;
                }
                let selected = listview_ptr.selection_model().selected_indexes();
                if selected.length() == 0 {
                    return;
                }
                for c in 0..selected.length() {
                    listview_ptr.model().remove_row_1a(c);
                }
            }});
            let f = Self {
                _main: main_ptr,
                model,
                mode_toolbar,
                //toolbar,
                view: listview_ptr,
                items: listitems,
                delete_shortcut: delete_shortcut.into_ptr(),
                rm: rm_slot,
            };
            //f.toolbar.rm_action.triggered().connect(&f.rm);
            f.delete_shortcut.activated().connect(&f.rm);
            f
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        unsafe {
            self.items.borrow_mut().clear();
            self.model.clear();
        }
    }

    #[allow(dead_code)]
    /// Sets the contents to items.
    pub fn set_items<'a: 'l, I>(&mut self, items: Vec<I>)
    where
        I: Into<&'a str>,
    {
        unsafe {
            self.items.borrow_mut().clear();
            self.model.clear();
            for item in items {
                self.add_item(item.into());
            }
        }
    }

    /// add an item to the pulldown
    pub fn add_item<'a, I>(&mut self, item: I)
    where
        I: Into<&'a str>,
    {
        unsafe {
            self.items
                .borrow_mut()
                .add_item_to(item.into(), &mut self.model.as_mut_ptr());
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
    fn setup_main_widget(parent: &MutPtr<QWidget>) -> MutPtr<QWidget> {
        QWidget::create(&parent).add_layout(LayoutType::VBoxLayout)
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
            qlv.set_selection_mode(SelectionMode::ExtendedSelection);
            qlv.set_drag_drop_overwrite_mode(false);
            qlv.set_drag_drop_mode(DragDropMode::InternalMove);
            let qlv_ptr = qlv.as_mut_ptr();
            layout.add_widget(qlv.into_ptr());
            qlv_ptr
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use qt_widgets::QApplication;
//     #[test]
//     fn can_create_new_item_list() {
//         unsafe {
//             let app = QApplication::new_2a()
//             let mut parent = QWidget::new_0a();
//             let layout = QVBoxLayout::new_0a();
//             parent.set_layout(layout.into_ptr());
//             let widget = QWidget::create(&parent.as_mut_ptr());
//             assert!(!widget.is_null());
//         }
//     }
// }
