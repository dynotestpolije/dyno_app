use std::borrow::Cow;

use dyno_types::DynoResult;
use eframe::egui::{mutex::Mutex, pos2, ColorImage, Rect, TextureHandle, TextureOptions};

// ----------------------------------------------------------------------------

#[cfg(feature = "images")]
pub type ImgFmt = image::ImageFormat;

#[derive(Default)]
pub struct Img {
    name: Cow<'static, str>,
    image: Mutex<ColorImage>,
    texture: Mutex<Option<TextureHandle>>,
}

impl Img {
    const UV: Rect = Rect::from_min_max(pos2(0f32, 0f32), pos2(1f32, 1f32));

    #[inline]
    pub fn from_color_image<C: Into<Cow<'static, str>>>(name: C, image: ColorImage) -> Self {
        Self {
            name: name.into(),
            image: Mutex::new(image),
            texture: Default::default(),
        }
    }

    #[cfg(feature = "images")]
    #[inline]
    pub fn from_image_bytes<C: Into<Cow<'static, str>>>(
        name: C,
        image_bytes: &[u8],
    ) -> DynoResult<Self> {
        Ok(Self::from_color_image(name, load_image_bytes(image_bytes)?))
    }

    #[cfg(feature = "images")]
    #[inline]
    pub fn from_image_bytes_format<C: Into<Cow<'static, str>>>(
        name: C,
        image_bytes: &[u8],
        format: ImgFmt,
    ) -> DynoResult<Self> {
        Ok(Self::from_color_image(
            name,
            load_image_bytes_format(image_bytes, format)?,
        ))
    }

    #[inline]
    pub fn size(&self) -> [usize; 2] {
        self.image.lock().size
    }
    #[inline]
    pub fn width(&self) -> usize {
        self.size()[0]
    }

    /// The height of the image.
    #[inline]
    pub fn height(&self) -> usize {
        self.size()[1]
    }

    #[inline]
    pub fn size_vec2(&self) -> eframe::egui::Vec2 {
        let [x, y] = self.size();
        eframe::egui::Vec2 {
            y: y as _,
            x: x as _,
        }
    }

    pub fn texture_id(&self, ctx: &eframe::egui::Context) -> eframe::egui::TextureId {
        let Self {
            name,
            image,
            texture,
        } = self;
        let inserts = || {
            let image: ColorImage = {
                let mut locked = image.lock();
                std::mem::take(&mut locked)
            };
            ctx.load_texture(name.to_string(), image, TextureOptions::LINEAR)
        };
        texture.lock().get_or_insert_with(inserts).id()
    }

    pub fn show_max_size(
        &self,
        ui: &mut eframe::egui::Ui,
        max_size: eframe::egui::Vec2,
    ) -> eframe::egui::Response {
        let mut desired_size = self.size_vec2();
        desired_size *= (max_size.x / desired_size.x).min(1.0);
        desired_size *= (max_size.y / desired_size.y).min(1.0);
        self.show_size(ui, desired_size)
    }
    pub fn show(&self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        self.show_size(ui, self.size_vec2())
    }
    /// Show the image with the given scale factor (1.0 = original size).
    pub fn show_scaled(&self, ui: &mut eframe::egui::Ui, scale: f32) -> eframe::egui::Response {
        self.show_size(ui, self.size_vec2() * scale)
    }
    /// Show the image with the given size.
    pub fn show_size(
        &self,
        ui: &mut eframe::egui::Ui,
        desired_size: eframe::egui::Vec2,
    ) -> eframe::egui::Response {
        ui.image(self.texture_id(ui.ctx()), desired_size)
    }

    #[allow(unused)]
    #[inline]
    pub fn get_shape(
        &self,
        ui: &mut eframe::egui::Ui,
        rect: eframe::egui::Rect,
    ) -> eframe::egui::Shape {
        use eframe::epaint::{pos2, Color32, Rect, Shape};
        Shape::image(self.texture_id(ui.ctx()), rect, Self::UV, Color32::WHITE)
    }
}

// ----------------------------------------------------------------------------

#[cfg(feature = "images")]
pub fn load_image_bytes(image_bytes: &[u8]) -> DynoResult<ColorImage> {
    let image = image::load_from_memory(image_bytes)
        .map_err(|err| format!("Failed to Load Image from Static Memory: {err}"))?;
    let size = [image.width() as _, image.height() as _];
    let rgba = image.to_rgba8();
    let pixels = rgba.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

#[cfg(feature = "images")]
pub fn load_image_bytes_format(image_bytes: &[u8], format: ImgFmt) -> Result<ColorImage, String> {
    let image = image::load_from_memory_with_format(image_bytes, format)
        .map_err(|err| format!("Failed to Load formated Image from Static Memory: {err}"))?;
    let size = [image.width() as _, image.height() as _];
    let rgba = image.to_rgba8();
    let pixels = rgba.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}
