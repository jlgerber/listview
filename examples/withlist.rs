use listitem::{
    //withlist::WithList,
    item_list::ItemList,
    utility::{create_vlayout, qs},
};
use qt_core::{QResource, Slot};
use qt_gui::QKeySequence;
use qt_widgets::{QApplication, QShortcut, QWidget};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    QApplication::init(|_app| unsafe {
        let _result = QResource::register_resource_q_string(&qs("/Users/jgerber/bin/withlist.rcc"));
        let mut main = QWidget::new_0a();
        let mut main_ref = main.as_mut_ptr();
        let main_layout = create_vlayout();

        main.set_layout(main_layout.into_ptr());

        let item_list = Rc::new(RefCell::new(ItemList::new(&mut main_ref)));

        let wl_c1 = item_list.clone();
        let wl_c2 = item_list.clone();
        item_list
            .borrow_mut()
            .set_stylesheet("/Users/jgerber/bin/withlist.qss");

        let find_slot: Slot<'static> = Slot::new(move || {
            wl_c1.borrow_mut().set_find_mode();
        });

        let add_slot: Slot<'static> = Slot::new(move || {
            wl_c2.borrow_mut().set_add_mode();
        });

        let key_seq = QKeySequence::from_q_string(&qs("Ctrl+f"));
        let find_shortcut = QShortcut::new_2a(key_seq.as_ref(), item_list.borrow_mut().main);

        let key_seq = QKeySequence::from_q_string(&qs("Ctrl+a"));
        let add_shortcut = QShortcut::new_2a(key_seq.as_ref(), item_list.borrow_mut().main);

        item_list.borrow_mut().set_cb_items(vec![
            "amtools",
            "animcomp",
            "animpublish",
            "animrender",
            "assetbrowser",
            "assetmanager",
            "atomic",
            "autorender",
            "dd",
            "ddg",
            "deferredpipeline",
            "gcc",
            "houdini",
            "houdinipipeline",
            "houdinisubmission",
            "jsconfig",
            "jstools",
            "jsutils",
            "layoutpipelne",
            "lightpipeline",
            "make",
            "mari",
            "maya",
            "modelpipeline",
            "modelpublish",
            "mudbox",
            "nuke",
            "nukesubmission",
            "organic",
            "packaboo",
            "packaboo_utils",
            "packrat",
            "pk",
            "pbutils",
            "prez",
            "qt",
            "qtpy",
            "race",
            "racetrack",
            "raceview",
            "redshift",
            "rigtools",
            "samson",
            "shotgun",
            "shotgunapi",
            "submission",
            "texturepublish",
            "texturepipeline",
            "vray",
            "vrayddbase",
            "vray_for_maya",
            "wam",
            "wambase",
            "xerces",
        ]);
        find_shortcut.activated().connect(&find_slot);
        add_shortcut.activated().connect(&add_slot);

        item_list.borrow_mut().set_add_mode();

        main_ref.show();

        QApplication::exec()
    });
}
