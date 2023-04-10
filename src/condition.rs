use dyno_types::paste::paste;
use dynotest_app::widgets::button::ButtonExt;

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FileType {
    All,
    Binaries,
    Csv,
    Excel,
}

impl FileType {
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        match self {
            FileType::All => "All",
            FileType::Binaries => "Binaries",
            FileType::Csv => "Csv",
            FileType::Excel => "Excel",
        }
    }
    pub fn path<P>(self, parent: P) -> std::path::PathBuf
    where
        P: AsRef<std::path::Path>,
    {
        parent.as_ref().join(self.as_str())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum OperatorData {
    Noop,
    SaveFile(FileType),
    OpenFile(FileType),
}

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    operator: OperatorData,

    show_help: bool,
    show_about: bool,
    show_config: bool,
    buffer_saved: bool,
    confirm_reload: bool,
    confirm_quit: bool,

    allow_close: bool,
}

impl Default for Condition {
    fn default() -> Self {
        Self {
            operator: OperatorData::Noop,
            show_help: false,
            show_about: false,
            show_config: false,
            buffer_saved: false,
            confirm_reload: false,
            confirm_quit: false,
            allow_close: false,
        }
    }
}

macro_rules! impl_cond_and {
    ($($name:ident: $tp:ty => $def:expr),*) => {
        impl Condition {
            paste!($(
                #[allow(unused)]
                #[inline(always)]
                pub fn [<$name _and>](&mut self, callback: impl FnOnce(&mut $tp) -> ()) {
                    if self.$name == $def {
                        return;
                    }
                    callback(&mut self.$name);
                }
            )*);
        }
    };
}
macro_rules! impl_cond_setter_getter {
    ($($name:ident: $tp:ty => $def:expr),*) => {
        impl Condition {
            paste!($(
                #[allow(unused)]
                #[inline(always)]
                pub fn [<get_ $name>](&self) -> $tp {
                    self.$name
                }

                #[allow(unused)]
                #[inline(always)]
                pub fn [<get_ $name _mut>](&mut self) -> &mut $tp {
                    &mut self.$name
                }

                #[allow(unused)]
                #[inline(always)]
                pub fn [<set_ $name>](&mut self, val: $tp) {
                    self.$name = val;
                }
            )*);
        }
    };
}

macro_rules! impl_cond_all {
    ($($tok:tt)*) => {
        impl_cond_and!($($tok)*);
        impl_cond_setter_getter!($($tok)*);
    };
}

impl_cond_all!(
    operator: OperatorData => OperatorData::Noop,
    show_help: bool => true,
    show_about: bool => true,
    show_config: bool => true,
    buffer_saved: bool => true,
    confirm_quit: bool => true,
    confirm_reload: bool => true,
    allow_close: bool => true
);

impl Condition {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn menubar(&mut self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("File", |menu_ui| {
            if menu_ui.open_button().clicked() {
                self.operator = OperatorData::OpenFile(FileType::Binaries);
            }
            menu_ui.menu_button("Open As..", |submenu_ui| {
                if submenu_ui.button("Csv File").clicked() {
                    self.operator = OperatorData::OpenFile(FileType::Csv);
                }
                if submenu_ui.button("Excel File").clicked() {
                    self.operator = OperatorData::OpenFile(FileType::Excel);
                }
                if submenu_ui.button("Binaries File").clicked() {
                    self.operator = OperatorData::OpenFile(FileType::Binaries);
                }
            });
            if menu_ui.save_button().clicked() {
                self.operator = OperatorData::SaveFile(FileType::Binaries);
            }
            menu_ui.menu_button("Save As..", |submenu_ui| {
                if submenu_ui.button("Csv File").clicked() {
                    self.operator = OperatorData::SaveFile(FileType::Csv);
                }
                if submenu_ui.button("Excel File").clicked() {
                    self.operator = OperatorData::SaveFile(FileType::Excel);
                }
                if submenu_ui.button("Binaries File").clicked() {
                    self.operator = OperatorData::SaveFile(FileType::Binaries);
                }
            });
            if menu_ui.button("Reload").clicked() {
                self.confirm_reload = !self.confirm_reload;
            }
            if menu_ui.button("Quit").clicked() {
                self.confirm_quit = !self.confirm_quit;
            }
        });
        if ui.button("Config").clicked() {
            self.show_config = !self.show_config;
        }
        if ui.button("Help").clicked() {
            self.show_config = !self.show_config;
        }
        if ui.button("About").clicked() {
            self.show_about = !self.show_about;
        }
    }
}
