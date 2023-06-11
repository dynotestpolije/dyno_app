use dyno_core::AsStr;
use dyno_core::{paste::paste, serde};

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub enum DynoFileType {
    Dyno,
    Csv,
    Excel,
}

impl std::fmt::Display for DynoFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsStr<'static> for DynoFileType {
    #[inline]
    fn as_str(&self) -> &'static str {
        match self {
            DynoFileType::Dyno => "Binaries",
            DynoFileType::Csv => "Csv",
            DynoFileType::Excel => "Excel",
        }
    }
}

impl DynoFileType {
    pub fn path<P>(self, parent: P) -> std::path::PathBuf
    where
        P: AsRef<std::path::Path>,
    {
        parent.as_ref().join(self.as_str())
    }
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "dyno" | "dbin" => Some(Self::Dyno),
            "csv" | "dynocsv" => Some(Self::Csv),
            "xlsx" => Some(Self::Excel),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub enum OperatorData {
    #[default]
    Noop,
    SaveFile(DynoFileType),
    OpenFile(DynoFileType),
}
impl OperatorData {
    pub fn save_default() -> Self {
        Self::SaveFile(DynoFileType::Dyno)
    }

    #[inline]
    pub fn take(&mut self) -> Self {
        let ret = *self;
        *self = Self::Noop;
        ret
    }
}

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub struct DynoState {
    #[serde(skip)]
    operator: OperatorData,
    show_bottom_panel: bool,
    show_left_panel: bool,
    show_logger_window: bool,

    #[serde(skip)]
    quitable: bool,
    #[serde(skip)]
    quit: bool,
}

impl Default for DynoState {
    fn default() -> Self {
        Self {
            operator: OperatorData::Noop,
            show_logger_window: false,
            show_bottom_panel: true,
            show_left_panel: true,
            quitable: false,
            quit: false,
        }
    }
}

impl_cond_all!(
    show_left_panel     : bool => false,
    show_logger_window  : bool => false,
    show_bottom_panel   : bool => false,
    quitable            : bool => false,
    quit                : bool => false,
);

impl DynoState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_operator(&mut self) -> OperatorData {
        self.operator.take()
    }
    pub fn set_operator(&mut self, op: OperatorData) {
        self.operator = op;
    }
}

impl DynoState {
    pub fn key_events(&mut self, ctx: &eframe::egui::Context) {
        use eframe::egui::{Key, Modifiers};
        ctx.input_mut(|i| {
            if i.consume_key(
                Modifiers {
                    ctrl: true,
                    shift: true,
                    ..Default::default()
                },
                Key::S,
            ) {
                self.operator = OperatorData::save_default();
            }
        });
    }
}

macro_rules! impl_cond_and {
    ($($name:ident: $tp:ty => $def:expr),* $(,)?) => {
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
    ($($name:ident: $tp:ty => $def:expr),* $(,)?) => {
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

                #[allow(unused)]
                #[inline(always)]
                pub fn [<swap_ $name>](&mut self) {
                    self.$name = !self.$name;
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
