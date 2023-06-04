pub mod button;
pub mod hyperlink;
pub mod images;
pub mod msgdialog;
pub mod popup;
pub mod toast;

pub mod segment_display;

mod common;
mod gauges;
mod opener;
mod realtime_plot;

pub use gauges::{Gauge, GaugePreset, GaugeTypes};
pub use opener::{DynoFileManager, Filters};
pub use realtime_plot::RealtimePlot;

pub use common::*;

#[macro_export]
macro_rules! row_label_value {
    ($ui:ident, $v:ident, $name:literal, $desc:literal $(,)?) => {
        $ui.link($name).on_hover_ui(|hover_ui| {
            hover_ui.label(format!(
                "Value for `{}` type of `{}`",
                $name,
                $v.name_type()
            ));
            hover_ui.monospace($desc);
        });
        $ui.label($v.to_string());
    };
    ($ui:ident, $v:expr, $name:literal, $desc:literal $(,)?) => {
        $ui.link($name).on_hover_ui(|hover_ui| {
            hover_ui.label(concat!("Value for ", $name));
            hover_ui.monospace($desc);
        });
        $ui.add($v)
    };
    ($ui:ident => $v:expr, $name:literal, $desc:literal $(,)?) => {
        $ui.link($name).on_hover_ui(|hover_ui| {
            hover_ui.label(concat!("Value for ", $name));
            hover_ui.monospace($desc);
        });
        $v
    };
}
