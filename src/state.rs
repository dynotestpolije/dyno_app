use crate::widgets::button::ButtonExt as _;
use dyno_types::paste::paste;

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum DynoFileType {
    All,
    Binaries,
    Csv,
    Excel,
}

impl DynoFileType {
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        match self {
            DynoFileType::All => "All",
            DynoFileType::Binaries => "Binaries",
            DynoFileType::Csv => "Csv",
            DynoFileType::Excel => "Excel",
        }
    }
    pub fn path<P>(self, parent: P) -> std::path::PathBuf
    where
        P: AsRef<std::path::Path>,
    {
        parent.as_ref().join(self.as_str())
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum OperatorData {
    #[default]
    Noop,
    SaveFile(DynoFileType),
    OpenFile(DynoFileType),
}
impl OperatorData {
    pub fn save_all() -> Self {
        Self::SaveFile(DynoFileType::All)
    }

    #[inline]
    pub fn take(&mut self) -> Self {
        let ret = *self;
        *self = Self::Noop;
        ret
    }
}

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct DynoState {
    #[serde(skip)]
    operator: OperatorData,

    show_help: bool,
    show_about: bool,
    show_config: bool,
    show_bottom_panel: bool,
    show_left_panel: bool,
    show_logger_window: bool,
    show_buffer_unsaved: bool,

    quitable: bool,
}

impl Default for DynoState {
    fn default() -> Self {
        Self {
            operator: OperatorData::Noop,
            show_help: false,
            show_about: false,
            show_config: false,
            show_logger_window: false,
            show_bottom_panel: true,
            show_left_panel: true,
            show_buffer_unsaved: false,
            quitable: false,
        }
    }
}

impl_cond_all!(
    show_help           : bool => false,
    show_about          : bool => false,
    show_config         : bool => false,
    show_left_panel     : bool => false,
    show_logger_window  : bool => false,
    show_bottom_panel   : bool => false,
    show_buffer_unsaved : bool => false,
    quitable            : bool => false
);

impl DynoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_operator(&mut self) -> OperatorData {
        if self.show_buffer_unsaved {
            return OperatorData::Noop;
        }
        self.operator.take()
    }
    pub fn set_operator(&mut self, op: OperatorData) {
        self.operator = op;
    }
}

impl DynoState {
    #[inline]
    pub fn menubar(&mut self, ui: &mut eframe::egui::Ui) {
        use dyno_types::log as LOG;
        ui.menu_button("File", |menu_ui| {
            if menu_ui.open_button().clicked() {
                LOG::info!("Open Button menu clicked");
                self.operator = OperatorData::OpenFile(DynoFileType::Binaries);
            }
            menu_ui.menu_button("Open As..", |submenu_ui| {
                if submenu_ui.button("Csv File").clicked() {
                    self.operator = OperatorData::OpenFile(DynoFileType::Csv);
                    LOG::info!("Open as Csv file submenu clicked");
                }
                if submenu_ui.button("Excel File").clicked() {
                    self.operator = OperatorData::OpenFile(DynoFileType::Excel);
                    LOG::info!("Open as Excel file submenu clicked");
                }
                if submenu_ui.button("Binaries File").clicked() {
                    self.operator = OperatorData::OpenFile(DynoFileType::Binaries);
                    LOG::info!("Open as Binaries file submenu clicked");
                }
            });
            if menu_ui.save_button().clicked() {
                LOG::info!("Save file menu clicked");
                self.operator = OperatorData::SaveFile(DynoFileType::Binaries);
            }
            menu_ui.menu_button("Save As..", |submenu_ui| {
                if submenu_ui.button("Csv File").clicked() {
                    LOG::info!("Save as Csv file submenu clicked");
                    self.operator = OperatorData::SaveFile(DynoFileType::Csv);
                }
                if submenu_ui.button("Excel File").clicked() {
                    LOG::info!("Save as Excel file submenu clicked");
                    self.operator = OperatorData::SaveFile(DynoFileType::Excel);
                }
                if submenu_ui.button("Binaries File").clicked() {
                    LOG::info!("Save as Binaries file submenu clicked");
                    self.operator = OperatorData::SaveFile(DynoFileType::Binaries);
                }
            });
            if menu_ui.button("Quit").clicked() {
                LOG::info!("Exit submenu clicked");
                self.quitable = !self.quitable;
            }
        });
        ui.menu_button("View", |submenu_ui| {
            submenu_ui.checkbox(&mut self.show_bottom_panel, "Bottom Panel");
            submenu_ui.checkbox(&mut self.show_left_panel, "Left Panel");
            submenu_ui.checkbox(&mut self.show_logger_window, "Logger Window");
        });
        if ui.button("Config").clicked() {
            LOG::info!("Config submenu clicked");
            self.show_config = !self.show_config;
        }
        if ui.button("Help").clicked() {
            LOG::info!("Help submenu clicked");
            self.show_config = !self.show_config;
        }
        if ui.button("About").clicked() {
            LOG::info!("About submenu clicked");
            self.show_about = !self.show_about;
        }
    }
}

macro_rules! impl_cond_and {
    ($($name:ident: $tp:ty => $def:expr),*) => {
        impl DynoState {
            paste!($(
                #[allow(unused)]
                #[inline(always)]
                pub fn [<$name _and>](&mut self, callback: impl FnOnce(&mut $tp)) {
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
        impl DynoState {
            paste!($(
                #[allow(unused)]
                #[inline(always)]
                pub fn $name(&self) -> $tp {
                    self.$name
                }

                #[allow(unused)]
                #[inline(always)]
                pub fn [<$name _mut>](&mut self) -> &mut $tp {
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

use impl_cond_all;
use impl_cond_and;
use impl_cond_setter_getter;
