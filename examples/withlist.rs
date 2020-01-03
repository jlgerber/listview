use qt_core::QResource;
use qt_widgets::QApplication;

use listitem::{utility::qs, withlist::WithList};

fn main() {
    QApplication::init(|_app| unsafe {
        let _result = QResource::register_resource_q_string(&qs("/Users/jgerber/bin/withlist.rcc"));

        let mut with_list = WithList::new();
        with_list.set_stylesheet("/Users/jgerber/bin/withlist.qss");

        with_list.add_items(vec![
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

        with_list.item_list.borrow_mut().set_reorder_mode();

        with_list.show();

        QApplication::exec()
    });
}
