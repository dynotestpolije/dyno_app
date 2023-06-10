#[macro_export]
macro_rules! get_assets {
    ($asset: literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $asset))
    };
}

pub const APP_KEY: &str = "dynotest-app";

pub mod assets {
    use dyno_core::lazy_static;

    macro_rules! declare_static_ico {
        ($($name: ident => $file_name: literal),* $(,)?) => {$(
            match image::load_from_memory_with_format(
                $crate::get_assets!($file_name),
                image::ImageFormat::Ico
            ) {
                Ok(data) => {
                    let image = data.to_rgba8();
                    let width= image.width();
                    let height= image.height();
                    Some((IconData { rgba: image.into_raw(), width, height, }, [width, height]))
                },
                Err(err) => {
                    if !$crate::msg_dialog_err!(
                        OkReport => ["ignore the error", "report the error to developer"],
                        "error loading an assets",
                        "failed to load icon {} because: {}",
                        stringify!($name), err
                    ) {
                        todo!("Report th error")
                    } else {
                        None
                    }
                }
            }
        )*};
    }

    use eframe::IconData;
    lazy_static::lazy_static! {
        pub static ref ICO_LOGO: Option<(IconData, [u32; 2])> = declare_static_ico!(ICO_LOGO  => "icons/polije.ico");
        pub static ref ICO_ERROR: Option<(IconData, [u32; 2])> = declare_static_ico!(ICO_ERROR => "icons/error.ico");
        pub static ref ICO_WARN: Option<(IconData, [u32; 2])> = declare_static_ico!(ICO_WARN => "icons/warning.ico");
        pub static ref ICO_INFO: Option<(IconData, [u32; 2])> = declare_static_ico!(ICO_INFO => "icons/info.ico");
    }

    macro_rules! declare_static_img {
        ($($name: ident: $format:ident => $file_name: literal),* $(,)?) => {$(
            match Img::from_image_bytes_format(
                stringify!($name),
                $crate::get_assets!($file_name),
                ImgFmt::$format
            ) {
                Ok(img) => img,
                Err(err) => if !$crate::msg_dialog_err!(
                    OkReport => [ "Ignore the Error", "Report the error to Developer" ],
                    "ERROR Running Applications", "Failed to load icon {} because: {}",
                    stringify!($name), err
                    ) {
                        todo!("report")
                    } else {
                        Default::default()
                }
            }
        )*};
    }

    use crate::widgets::images::{Img, ImgFmt};
    lazy_static::lazy_static! {
        pub static ref POLIJE_LOGO_PNG:         Img = declare_static_img!(POLIJE_LOGO_PNG:          Png => "logo-512x512.png");
        pub static ref COLORIMAGE_GAUGE_RPM:    Img = declare_static_img!(COLORIMAGE_GAUGE_RPM:     Png => "gauge/dynotest_gauge_rpm-256.png");
        pub static ref COLORIMAGE_GAUGE_SPEED:  Img = declare_static_img!(COLORIMAGE_GAUGE_SPEED:   Png => "gauge/dynotest_gauge_speed-256.png");
        pub static ref COLORIMAGE_GAUGE_TORQUE: Img = declare_static_img!(COLORIMAGE_GAUGE_TORQUE:  Png => "gauge/dynotest_gauge_torque-256.png");
        pub static ref COLORIMAGE_GAUGE_HP:     Img = declare_static_img!(COLORIMAGE_GAUGE_HP:      Png => "gauge/dynotest_gauge_hp-256.png");
    }

    #[macro_export]
    macro_rules! open_option_icon {
        ($path:expr) => {{
            match image::open(&$path) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    Some(IconData {
                        rgba: rgba.into_raw(),
                        width,
                        height,
                    })
                }
                Err(err) => {
                    dyno_core::log::error!(
                        "failed to load image in path: {} - {}",
                        $path.display(),
                        err
                    );
                    None
                }
            }
        }};
    }
}

// ----------------------------------------------------------------------------
macro_rules! option_env_some {
    ( $x:expr ) => {
        match option_env!($x) {
            Some(env) if env.is_empty() => None,
            opt => opt,
        }
    };
    ( $x:expr, $def:expr ) => {
        match option_env!($x) {
            Some(env) if env.is_empty() => $def,
            Some(env) => env,
            None => $def,
        }
    };
}

#[doc(hidden)]
pub struct PackageInfo<'a> {
    pub app_name: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub authors: &'a str,
    pub description: Option<&'a str>,
    pub homepage: Option<&'a str>,
    pub repository: Option<&'a str>,
    pub license: Option<&'a str>,
    pub license_file: Option<&'a str>,
}

impl PackageInfo<'static> {
    pub const fn new() -> PackageInfo<'static> {
        Self {
            app_name: option_env_some!("APPLICATION_NAME_PRETTY", "DynoTest Application"),
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            authors: env!("CARGO_PKG_AUTHORS"),
            description: option_env_some!("CARGO_PKG_DESCRIPTION"),
            homepage: option_env_some!("CARGO_PKG_HOMEPAGE"),
            repository: option_env_some!("CARGO_PKG_REPOSITORY"),
            license: option_env_some!("CARGO_PKG_LICENSE"),
            license_file: option_env_some!("CARGO_PKG_LICENSE_FILE"),
        }
    }
    pub fn authors(&'static self) -> impl Iterator<Item = (&'static str, Option<&'static str>)> {
        self.authors.split(':').map(|author_line| {
            let author_parts = author_line
                .split(|c| ['<', '>'].contains(&c))
                .map(str::trim)
                .collect::<Vec<_>>();
            (author_parts[0], author_parts.get(1).copied())
        })
    }
}

pub static PACKAGE_INFO: PackageInfo<'static> = PackageInfo::new();

// ----------------------------------------------------------------------------
