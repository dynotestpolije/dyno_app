use dyno_types::DynoResult;
use rfd::{MessageButtons, MessageDialog, MessageLevel};

pub enum Level {
    Info,
    Warn,
    Erro,
}

impl From<Level> for MessageLevel {
    #[inline(always)]
    fn from(value: Level) -> Self {
        match value {
            Level::Info => Self::Info,
            Level::Warn => Self::Warning,
            Level::Erro => Self::Error,
        }
    }
}

pub enum Button {
    Ok,
    OkCancel,
    YesNo,
    OkReport,
    OkIgnore,
}
impl Button {
    pub fn desc(self, desc: [&str; 2]) -> String {
        match self {
            Button::Ok => format!("click `Ok` to {}", desc[0]),
            Button::OkCancel => format!(
                "click `Ok` to {}, or click `Cancel` to {}",
                desc[0], desc[1]
            ),
            Button::YesNo => format!("click `Yes` to {}, or click `No` to {}", desc[0], desc[1]),
            Button::OkReport => format!(
                "click `Ok` to {}, or click `Report` to Report to the Developer",
                desc[0]
            ),
            Button::OkIgnore => format!(
                "click `Ok` to {}, or click `Ignore` to Report to Ignore the Dialog",
                desc[0]
            ),
        }
    }
}

impl From<Button> for MessageButtons {
    fn from(value: Button) -> Self {
        match value {
            Button::Ok => MessageButtons::Ok,
            Button::OkCancel => MessageButtons::OkCancel,
            Button::YesNo => MessageButtons::YesNo,
            Button::OkReport => MessageButtons::OkCustom("Report".to_owned()),
            Button::OkIgnore => MessageButtons::OkCustom("Ignore".to_owned()),
        }
    }
}

pub fn msg_dialog_impl<S, D>(title: S, desc: D, level: Level, buttons: Button) -> bool
where
    S: AsRef<str>,
    D: AsRef<str>,
{
    MessageDialog::new()
        .set_title(title.as_ref())
        .set_description(desc.as_ref())
        .set_level(level.into())
        .set_buttons(buttons.into())
        .show()
}

#[macro_export]
macro_rules! msg_dialog{
    ($btn:expr => [$($desc:expr),*], $lvl:expr, $name:expr,  $($err:tt)*) => {{
        let desc = format!("{}\n{}", format!($($err)*), $btn.desc([$($desc),*]));
        $crate::widgets::msgdialog::msg_dialog_impl(
            $name, desc,
            $lvl,
            $btn,
        )
    }};
}

#[macro_export]
macro_rules! msg_dialog_err {
    ($btn:ident => [$($desc:expr),*], $name:expr, $($err:tt)*) => {
        $crate::msg_dialog!($crate::widgets::msgdialog::Button::$btn => [$($desc),*], $crate::widgets::msgdialog::Level::Erro, $name, $($err)*)
    };
}

#[macro_export]
macro_rules! msg_dialog_warn {
    ($btn:ident => [$($desc:expr),+], $name:expr, $($err:tt)*) => {
        $crate::msg_dialog!($crate::widgets::msgdialog::Button::$btn => [$($desc),*], $crate::widgets::msgdialog::Level::Warn, $name, $($err)*)
    };
}
#[macro_export]
macro_rules! msg_dialog_info {
    ($btn:ident => [$($desc:expr),+], $name:expr, $($err:tt)*) => {
        $crate::msg_dialog!($crate::widgets::msgdialog::Button::$btn => [$($desc),*], $crate::widgets::msgdialog::Level::Info, $name, $($err)*)
    };
}

pub trait MsgDialogUnwrap<T> {
    fn msg_dialog_map(self, name: &'static str) -> Option<T>;
    fn msg_dialog_unwrap(self, name: &'static str) -> Option<T>;
    fn msg_dialog_unwrap_default(self, name: &'static str) -> T
    where
        T: Default;
}

impl<T> MsgDialogUnwrap<T> for DynoResult<'_, T> {
    #[inline(always)]
    fn msg_dialog_unwrap(self, name: &'static str) -> Option<T> {
        match self {
            Ok(k) => Some(k),
            Err(err) => {
                if !msg_dialog_err!(OkCancel => ["Ignore the Error", "Abort the Process and quit"], name, "{err}")
                {
                    panic!()
                }
                None
            }
        }
    }
    #[inline(always)]
    fn msg_dialog_unwrap_default(self, name: &'static str) -> T
    where
        T: Default,
    {
        match self {
            Ok(k) => k,
            Err(err) => {
                if msg_dialog_err!(Ok => ["Ignore the Error", ""], name, "{err}") {
                    Default::default()
                }
                Default::default()
            }
        }
    }
    #[inline(always)]
    fn msg_dialog_map(self, name: &'static str) -> Option<T> {
        self.msg_dialog_unwrap(name)
    }
}
