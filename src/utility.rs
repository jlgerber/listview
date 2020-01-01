use qt_core::QString;
use qt_core::{q_io_device::OpenModeFlag, QFile, QFlags, QTextStream};
use qt_widgets::cpp_core::CppBox;

use qt_widgets::{cpp_core::MutPtr, QWidget};

/// Given an input of &str or String, return a boxed QString
pub fn qs<S: AsRef<str>>(input: S) -> CppBox<QString> {
    QString::from_std_str(input.as_ref())
}

pub fn load_stylesheet(sheet: &str, mut widget: MutPtr<QWidget>) {
    unsafe {
        let mut file = QFile::from_q_string(&QString::from_std_str(sheet));
        if file.open_1a(QFlags::from(OpenModeFlag::ReadOnly)) {
            let mut text_stream = QTextStream::new();
            text_stream.set_device(file.as_mut_ptr());
            let stylesheet = text_stream.read_all();
            widget.set_style_sheet(stylesheet.as_ref());
        } else {
            log::warn!("stylesheet not found");
        }
    }
}
