use super::list_items::ListItems;
use super::utility::qs;
use log;
use qt_core::QSize;
use qt_core::ToolButtonStyle;
use qt_core::{Key, QString, Slot};
use qt_gui::{
    q_icon::{Mode, State},
    QIcon,
};
use qt_gui::{QKeySequence, QStandardItemModel};
use qt_widgets::q_abstract_item_view::DragDropMode;
use qt_widgets::{
    cpp_core::{CppBox, MutPtr, Ref},
    q_abstract_item_view::SelectionMode,
    q_action::ActionEvent,
    q_size_policy::Policy,
    QAction, QActionGroup, QComboBox, QFrame, QHBoxLayout, QLabel, QLayout, QListView, QShortcut,
    QSizePolicy, QToolBar, QToolButton, QVBoxLayout, QWidget,
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
/// Works like enclose but provides for both non and mutable
/// clones.
///
/// # Example
/// ```ignore
/// { enclose_all! { (<ARG>) (mut <ARG>,) move |<ARG>| {} }}
/// ```
/// EG
/// ```
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
// TRAITS
//

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

unsafe impl NewWidget<QWidget, QFrame> for QFrame {
    fn create(parent: &MutPtr<QWidget>) -> MutPtr<QFrame> {
        unsafe {
            let mut main = QFrame::new_0a();
            let main_ptr = main.as_mut_ptr();
            let mut parent_ptr = parent.layout();
            assert!(!parent_ptr.is_null());
            parent_ptr.add_widget(main.into_ptr());
            main_ptr
        }
    }
}

/// Choose the type of layout that you want to create
/// in the AddLayout trait implementation
#[allow(dead_code)]
pub enum LayoutType {
    VBoxLayout,
    HBoxLayout,
}

/// Trait provides a function to add a layout to
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

unsafe impl AddLayout<QFrame> for MutPtr<QFrame> {
    type Layout = LayoutType;

    fn add_layout(&mut self, layout: LayoutType) -> MutPtr<QFrame> {
        unsafe {
            let mut qw: MutPtr<QWidget> = self.static_upcast_mut();
            add_layout_to_widget(&mut qw, layout);
            self.as_mut_ref().unwrap().as_mut_ptr()
        }
    }
}

//
// ITEMLIST TOOLBAR
//
/// A struct holding pointers to the QToolbar instance,
/// along with the action group, all of the actions for the
/// buttons on the toolbar, as well as any internal slots
pub struct ItemListModeToolbar<'a> {
    pub toolbar: MutPtr<QToolBar>,
    pub action_group: MutPtr<QActionGroup>,
    pub reorder_mode_action: MutPtr<QAction>,
    pub rm_mode_action: MutPtr<QAction>,
    pub add_mode_action: MutPtr<QAction>,
    _mode_icon: CppBox<QIcon>,
    pub edit: Slot<'a>,
}

impl<'a> ItemListModeToolbar<'a> {
    /// New up an ItemListModeToolbar, and regiter it with it
    /// parent's layout, given it's parent widget.
    ///
    /// # Argument
    /// * `parent` - MutPtr wrapped QWidget
    ///
    /// # Returns
    /// * Instance of ItemListModelToolbar
    pub fn new(parent: &mut MutPtr<QWidget>) -> Self {
        unsafe {
            let mut toolbar = Self::create_toolbar("WithPackage Toolbar");
            let mut action_group = QActionGroup::new(toolbar.as_mut_ptr());
            let action_group_ptr = action_group.as_mut_ptr();
            // add spacer widget
            let spacer = Self::create_spacer();
            let mut mode_icon = QIcon::new();
            let size = QSize::new_2a(24, 24);
            mode_icon.add_file_4a(
                &qs(":images/radio_btn.svg"),
                &size,
                Mode::Normal,
                State::Off,
            );
            mode_icon.add_file_4a(
                &qs(":images/radio_btn_sel.svg"),
                &size,
                Mode::Normal,
                State::On,
            );

            // REORDER
            let (reorder_mode_action, _reorder_btn) = Self::create_mode_action(
                "Reorder",
                action_group_ptr,
                &mut toolbar.as_mut_ptr(),
                true,
                Some(mode_icon.as_ref()),
            );

            // REMOVE
            let (rm_mode_action, rm_button_ref) = Self::create_mode_action(
                "Remove",
                action_group_ptr,
                &mut toolbar.as_mut_ptr(),
                false,
                Some(mode_icon.as_ref()),
            );

            // ADD
            let (add_mode_action, _add_btn) = Self::create_mode_action(
                "Add",
                action_group_ptr,
                &mut toolbar.as_mut_ptr(),
                false,
                Some(mode_icon.as_ref()),
            );

            // add in spacer
            toolbar.add_widget(spacer.into_ptr());

            let toolbar_ptr = toolbar.as_mut_ptr();
            parent.layout().add_widget(toolbar.into_ptr());

            let edit = Slot::new(move || if rm_button_ref.is_enabled() {});

            let tb = Self {
                toolbar: toolbar_ptr,
                action_group: action_group.into_ptr(),
                //save_action: save_action,
                reorder_mode_action: reorder_mode_action.into_ptr(),
                rm_mode_action: rm_mode_action.into_ptr(),
                add_mode_action: add_mode_action.into_ptr(),
                _mode_icon: mode_icon,
                edit,
            };

            tb
        }
    }

    #[allow(dead_code)]
    /// Determine if the remove mode is active
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * bool indicating whether or not the remove mode is active
    pub fn is_remove_active(&self) -> bool {
        unsafe { self.rm_mode_action.is_checked() }
    }

    #[allow(dead_code)]
    /// Determine whether the add mode is active
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * bool indicating whether or not the add mode is active
    pub fn is_add_active(&self) -> bool {
        unsafe { self.add_mode_action.is_checked() }
    }

    #[allow(dead_code)]
    /// Determine whether the reorder mode is active
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * bool indicating whether the reorder is active
    pub fn is_reorder_active(&self) -> bool {
        unsafe { self.reorder_mode_action.is_checked() }
    }

    // Create and configure the QToolBar internal instance, provided a name
    //
    // # Arguments
    // * `name` - The proposed name of the new toolbar
    //
    // # Returns
    // * CppBoxed QToolBar instance
    unsafe fn create_toolbar(name: &str) -> CppBox<QToolBar> {
        let mut toolbar = QToolBar::from_q_string(&qs(name));
        toolbar.set_floatable(false);
        toolbar.set_movable(false);
        toolbar.set_object_name(&qs("WithsToolBar"));
        toolbar
    }

    // Create a widget that serves as a spacer for the toolbar.
    //
    // # Arguments
    // * None
    //
    // # Returns
    // * CppBoxed QWidget
    unsafe fn create_spacer() -> CppBox<QWidget> {
        let mut spacer = QWidget::new_0a();
        let sp = QSizePolicy::new_2a(Policy::Expanding, Policy::Fixed);
        spacer.set_size_policy_1a(sp.as_ref());
        spacer
    }

    #[allow(dead_code)]
    // Given a name, and the host toolbar, create and return an action.
    //
    // # Arguments
    // * `name` - The name of the action
    // * `toolbar` A mutable pointer to the QToolBar instance which will
    // host the action as a QToolButton
    //
    // # Returns tuple of
    // * New action,
    // * toolbutton that hosts the action on the toolbar
    unsafe fn create_action(
        name: &str,
        toolbar: &mut MutPtr<QToolBar>,
    ) -> (MutPtr<QAction>, MutPtr<QToolButton>) {
        let mode_action = toolbar.add_action_1a(&qs(name));
        let mut button: MutPtr<QToolButton> =
            toolbar.widget_for_action(mode_action).dynamic_cast_mut();
        button.set_object_name(&qs("WithsToolbarButton"));

        (mode_action, button)
    }

    #[allow(dead_code)]
    // Create a grouped action given a name, the group, toolbar, and an
    // indication of whether the action starts out checked. There should
    // be only one checked action per group.
    //
    // # Arguments
    // * `name` - The name of the action to be created
    // * `action_grp_ptr` - A pointer to the QActionGroup
    // * `toolbar` - A mutable reference to the MutPtr wrapped QToolbar instance
    // we wish to attach our action to
    // * `checked` - an indication of whether the action should be in the checked state
    //
    // # Returns Tuple of
    // * CppBoxed QAction instance created
    // * MutPtr wrapped QToolButton that hosts the action on the toolbar
    unsafe fn create_mode_action(
        name: &str,
        action_grp_ptr: MutPtr<QActionGroup>,
        toolbar: &mut MutPtr<QToolBar>,
        checked: bool,
        icon: Option<Ref<QIcon>>,
    ) -> (CppBox<QAction>, MutPtr<QToolButton>) {
        let mut mode_action = if let Some(icon) = icon {
            QAction::from_q_icon_q_string_q_object(icon, &qs(name), action_grp_ptr)
        } else {
            QAction::from_q_string_q_object(&qs(name), action_grp_ptr)
        };
        //let mut mode_action = QAction::from_q_string_q_object(&qs(name), action_grp_ptr);
        mode_action.set_checkable(true);
        mode_action.set_checked(checked);

        toolbar.add_action(mode_action.as_mut_ptr());

        let mut button: MutPtr<QToolButton> = toolbar
            .widget_for_action(mode_action.as_mut_ptr())
            .dynamic_cast_mut();

        button.set_object_name(&qs("WithsToolbarModeButton"));
        button.set_tool_button_style(ToolButtonStyle::ToolButtonTextBesideIcon);

        (mode_action, button)
    }
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
    pub _main: MutPtr<QWidget>,
    pub mode_toolbar: Rc<RefCell<ItemListModeToolbar<'l>>>,
    pub add_combobox: MutPtr<QComboBox>,
    pub model: CppBox<QStandardItemModel>,
    pub view: MutPtr<QListView>,
    pub items: Rc<RefCell<ListItems>>,
    pub enter_shortcut: MutPtr<QShortcut>,
    pub rm: Slot<'l>,
    pub reorder_mode: Slot<'l>,
    pub rm_mode: Slot<'l>,
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
            let mut main_ptr = Self::setup_main_widget(&parent);

            let listitems = Rc::new(RefCell::new(ListItems::new()));

            let mut model = Self::setup_model();
            let mut model_ptr = model.as_mut_ptr();

            let mode_toolbar = Rc::new(RefCell::new(ItemListModeToolbar::new(&mut main_ptr)));

            let cbox = Self::setup_combobox("ItemCombo", &mut main_ptr);
            let cbox_ptr = cbox.clone();

            let listview_ptr = Self::setup_listview(model.as_mut_ptr(), &mut main_ptr.layout());

            let key_seq = QKeySequence::from_int(Key::KeyReturn.to_int());
            let enter_shortcut = QShortcut::new_2a(key_seq.as_ref(), main_ptr);

            let rm_slot = Slot::new(enclose_all! { (mode_toolbar) (mut listview_ptr) move || {
                if !mode_toolbar.borrow().is_remove_active() {
                    return;
                }
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
                enclose_all! { (mode_toolbar) (mut listitems, mut cbox_ptr, mut listview_ptr) move || {
                    if !mode_toolbar.borrow().is_add_active() {return;}
                    let text = cbox_ptr.current_text();
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
                        log::info!("entry already exists");
                        return;
                    }

                    listitems.borrow_mut().add_item_to(text.to_std_string().as_str(),&mut model_ptr);
                    cbox_ptr.clear_edit_text();
                    listview_ptr.scroll_to_bottom();

                }},
            );

            let f = Self {
                _main: main_ptr,
                model,
                mode_toolbar,
                add_combobox: cbox,
                view: listview_ptr,
                items: listitems,
                enter_shortcut: enter_shortcut.into_ptr(),

                rm: rm_slot,

                reorder_mode: Slot::new(
                    enclose_all! {() (mut listview_ptr, mut cbox_ptr) move || {
                        listview_ptr.set_drag_drop_mode(DragDropMode::InternalMove);
                        listview_ptr.set_drag_enabled(true);
                        cbox_ptr.set_disabled(true);
                    }},
                ),

                rm_mode: Slot::new(enclose_all! { () (mut listview_ptr, mut cbox_ptr) move || {
                    listview_ptr.set_drag_enabled(false);
                    listview_ptr.set_drag_drop_mode(DragDropMode::NoDragDrop);
                    cbox_ptr.set_disabled(true);
                }}),

                add_mode: Slot::new(enclose_all! { () (mut listview_ptr, mut cbox_ptr) move || {
                    listview_ptr.set_drag_enabled(false);
                    listview_ptr.set_drag_drop_mode(DragDropMode::NoDragDrop);
                    cbox_ptr.set_enabled(true);
                }}),

                enter_sc,
            };

            f.mode_toolbar
                .borrow_mut()
                .reorder_mode_action
                .triggered()
                .connect(&f.reorder_mode);

            f.mode_toolbar
                .borrow_mut()
                .rm_mode_action
                .triggered()
                .connect(&f.rm_mode);

            f.mode_toolbar
                .borrow_mut()
                .add_mode_action
                .triggered()
                .connect(&f.add_mode);

            f.enter_shortcut.activated().connect(&f.enter_sc);

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
    ///
    /// # Arguments
    /// * The item to be added, as a &str or String
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

    #[allow(dead_code)]
    /// Set items in the combobox
    pub fn set_cb_items<'c, I>(&mut self, items: Vec<I>)
    where
        I: Into<&'c str>,
    {
        unsafe {
            self.remove_cb_items();
            self.add_combobox.add_item_q_string(&qs(""));
            for item in items {
                self.add_combobox.add_item_q_string(&qs(item.into()));
            }
        }
    }

    #[allow(dead_code)]
    /// Remove all items from the combobox
    pub fn remove_cb_items(&mut self) {
        unsafe {
            self.add_combobox.clear();
        }
    }

    pub fn set_reorder_mode(&mut self) {
        unsafe {
            self.mode_toolbar
                .borrow_mut()
                .reorder_mode_action
                .activate(ActionEvent::Trigger);
        }
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
    pub fn set_rm_mode(&mut self) {
        unsafe {
            self.mode_toolbar
                .borrow_mut()
                .rm_mode_action
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

    // Given a name and a parent, construct a QComboBox and return it
    //
    // #Arguments
    // * `name` - Name of the combobox
    // * `parent` - mut reference to the parent widget. Will be used to fetch the layout
    //
    // # Returns
    // * A MutPtr wrapping the QComboBox
    fn setup_combobox(name: &str, mut parent: &mut MutPtr<QWidget>) -> MutPtr<QComboBox> {
        unsafe {
            let mut cb_widget = QFrame::create(&mut parent);
            cb_widget.add_layout(LayoutType::HBoxLayout);
            cb_widget.set_object_name(&qs(format!("{}Widget", name)));

            let mut cb_label = QLabel::from_q_string(&qs("Add Item"));
            cb_label.set_object_name(&qs("WithsCBLabel"));
            cb_widget.layout().add_widget(cb_label.into_ptr());

            let mut cbox = QComboBox::new_0a();
            cbox.set_editable(true);
            cbox.set_object_name(&qs("WithsComboBox"));
            let cbox_ptr = cbox.as_mut_ptr();
            cb_widget.layout().add_widget(cbox.into_ptr());

            let mut layout = cb_widget.layout().dynamic_cast_mut::<QHBoxLayout>();
            if layout.is_null() {
                log::error!("unable to cast layout to QHBoxLayout");
                return cbox_ptr;
            }
            layout.set_stretch(1, 1);

            cbox_ptr
        }
    }
}
