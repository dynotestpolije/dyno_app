mod metrics;
mod segments;
mod widget;

use eframe::epaint::Pos2;
pub use metrics::{DisplayMetrics, DisplayMetricsPreset};
pub use segments::{NineSegment, SevenSegment, SixteenSegment};
pub use widget::SegmentedDisplay;

use dyno_core::derive_more::Display;

// ----------------------------------------------------------------------------

pub type DisplayGlyph = u16;

#[derive(Clone, Copy, Debug, Default)]
pub struct DisplayDigit {
    pub glyph: DisplayGlyph,
    pub dot: bool,
    pub colon: bool,
    pub apostrophe: bool,
}

// ----------------------------------------------------------------------------

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum DisplayKind {
    #[display(fmt = "7-segment")]
    SevenSegment,

    #[display(fmt = "9-segment")]
    NineSegment,

    #[display(fmt = "16-segment")]
    SixteenSegment,
}

impl DisplayKind {
    #[must_use]
    pub(crate) fn display_impl(&self) -> Box<dyn DisplayImpl> {
        match *self {
            DisplayKind::SevenSegment => Box::new(segments::SevenSegment),
            DisplayKind::NineSegment => Box::new(segments::NineSegment),
            DisplayKind::SixteenSegment => Box::new(segments::SixteenSegment),
        }
    }

    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.display_impl().segment_count()
    }
}

// ----------------------------------------------------------------------------

pub(crate) trait DisplayImpl {
    fn segment_count(&self) -> usize;

    fn glyph(&self, c: char) -> Option<DisplayGlyph>;

    fn geometry(
        &self,
        digit_width: f32,
        digit_height: f32,
        segment_thickness: f32,
        segment_spacing: f32,
        digit_median: f32,
    ) -> Vec<Vec<Pos2>>;
}
