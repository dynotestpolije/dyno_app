use std::{
    fs,
    ops::{Index, IndexMut},
    path::{Path, PathBuf},
};

use crate::widgets::DynoFileManager;
use dyno_core::{chrono::Local, paste::paste, serde, toml, CompresedSaver, DynoErr, DynoResult};
use serde::{de::DeserializeOwned, Serialize};

macro_rules! dyno_paths {
    ($struct_name: ident, [$($fn_name:ident),*]) => {
        impl $struct_name {
            paste! {$(
                #[allow(unused)]
                #[doc = "Create a wrapper function `" $fn_name "` with file name as parameter."]
                #[inline(always)]
                pub fn [<get_ $fn_name _file>](
                    &self,
                    file_name: impl AsRef<Path>
                ) -> PathBuf {
                    self.$fn_name.join(file_name)
                }

                #[allow(unused)]
                #[doc = "Create a wrapper function `" $fn_name "` with folder name as parameter."]
                #[inline(always)]
                pub fn [<get_ $fn_name _folder>](
                    &self,
                    folder_name: impl AsRef<Path>
                ) -> PathBuf {
                    self.$fn_name.join(folder_name)
                }
            )*}
        }
    };
}

// if !dirs.exists() {
//     fs::create_dir_all(&dirs)?;
// }

#[allow(unused)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub struct DynoPaths {
    pub name: String,
    pub project_path: PathBuf,

    // base directories
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub data_local_dir: PathBuf,
    pub preference_dir: PathBuf,
}

dyno_paths!(
    DynoPaths,
    [
        project_path,
        cache_dir,
        config_dir,
        data_dir,
        data_local_dir,
        preference_dir
    ]
);

#[inline]
pub fn file_name_timestamp(extension: &str) -> String {
    format!("{}.{}", Local::now().format("%v_%s"), extension)
}

impl Default for DynoPaths {
    fn default() -> Self {
        Self {
            name: "DynotestsApp".to_owned(),
            project_path: PathBuf::from("./DynotestsApp"),
            cache_dir: PathBuf::from("./DynotestsApp/cache"),
            config_dir: PathBuf::from("./DynotestsApp/config"),
            data_dir: PathBuf::from("./DynotestsApp/data"),
            data_local_dir: PathBuf::from("./DynotestsApp/local"),
            preference_dir: PathBuf::from("./DynotestsApp/preference"),
        }
    }
}

impl DynoPaths {
    pub fn new(name: &'static str) -> DynoResult<Self> {
        match Self::try_get_dirs(name).map(|p| p.into()) {
            Some(s) => Ok(s),
            None => Err(DynoErr::input_output_error(
                "Failed initialize Path, no valid home directory path could be retrieved from the operating system",
            )),
        }
    }

    pub fn check_is_changed(&mut self, other: &Self) {
        if crate::eq_structs!(self, other -> [
            name,
            project_path,
            cache_dir,
            config_dir,
            data_dir,
            data_local_dir,
            preference_dir
        ]) {
            *self = other.clone();
        }
    }

    #[inline(always)]
    fn try_get_dirs(name: &'static str) -> Option<directories::ProjectDirs> {
        directories::ProjectDirs::from("com", "PoliteknikNegeriJember", name)
    }
}

impl From<directories::ProjectDirs> for DynoPaths {
    fn from(pd: directories::ProjectDirs) -> Self {
        let project_path = pd.project_path().to_path_buf();
        let cache_dir = pd.cache_dir().to_path_buf();
        let config_dir = pd.config_dir().to_path_buf();
        let data_dir = pd.data_dir().to_path_buf();
        let data_local_dir = pd.data_local_dir().to_path_buf();
        let preference_dir = pd.preference_dir().to_path_buf();

        Self {
            name: String::from(crate::PACKAGE_INFO.app_name),
            project_path,
            cache_dir,
            config_dir,
            data_dir,
            data_local_dir,
            preference_dir,
        }
    }
}
impl Index<usize> for DynoPaths {
    type Output = PathBuf;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.project_path,
            1 => &self.cache_dir,
            2 => &self.config_dir,
            3 => &self.data_dir,
            4 => &self.data_local_dir,
            5 => &self.preference_dir,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<usize> for DynoPaths {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.project_path,
            1 => &mut self.cache_dir,
            2 => &mut self.config_dir,
            3 => &mut self.data_dir,
            4 => &mut self.data_local_dir,
            5 => &mut self.preference_dir,
            _ => unreachable!(),
        }
    }
}

impl DynoPaths {
    pub const PATHS_NAME: [&str; 6] = [
        r#"PROJECT PATH"#,
        r#"CACHE PATH"#,
        r#"CONFIG PATH"#,
        r#"DATA PATH"#,
        r#"DATA LOCAL PATH"#,
        r#"PREFERENCE PATH"#,
    ];

    #[inline]
    pub fn get_config<D>(&self, filename: &'_ str) -> DynoResult<D>
    where
        D: DeserializeOwned + Default + 'static,
    {
        let file = self.get_config_dir_file(filename);
        if !file.exists() {
            let err = format!("File config `{f}` doesn't exists!", f = file.display());
            return Err(DynoErr::input_output_error(err));
        }
        fs::read_to_string(file).map_or_else(
            |err| Err(From::from(err)),
            |x| toml::from_str(&x).map_err(From::from),
        )
    }

    #[inline]
    pub fn set_config<S>(&self, config: S, filename: &'_ str) -> DynoResult<()>
    where
        S: Serialize,
    {
        let file = self.get_config_dir_file(filename);
        let data = toml::to_string(&config)?;
        fs::write(file, data.as_bytes()).map_err(From::from)
    }

    #[inline]
    pub fn get_bin<D>(&self, filename: &'_ str) -> DynoResult<D>
    where
        D: CompresedSaver,
    {
        let file = self.get_data_dir_file(filename);
        if !file.exists() {
            return Err(DynoErr::filesystem_error(format!(
                "File binaries `{f}` doesn't exists!",
                f = file.display()
            )));
        }
        D::decompress_from_path(file).map_err(From::from)
    }

    #[inline]
    pub fn set_bin<S>(&self, config: S, filename: &'_ str) -> DynoResult<()>
    where
        S: CompresedSaver,
    {
        let file = self.get_data_dir_file(filename);
        S::compress_to_path(&config, file).map_err(From::from)
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> [&'_ mut PathBuf; 6] {
        [
            &mut self.project_path,
            &mut self.cache_dir,
            &mut self.config_dir,
            &mut self.data_dir,
            &mut self.data_local_dir,
            &mut self.preference_dir,
        ]
    }

    pub fn draw(&mut self, ui: &mut eframe::egui::Ui, edit: &mut bool) {
        use eframe::egui::{Grid, Link, TextEdit};
        ui.add_space(50.0);
        ui.add(TextEdit::singleline(&mut self.name).hint_text("app dir name"));
        ui.add_space(20.0);
        ui.checkbox(edit, "Edit Paths Config");
        ui.add_space(20.0);
        Grid::new("dyno_setting_paths")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (i, paths) in self.as_slice_mut().into_iter().enumerate() {
                    ui.label(Self::PATHS_NAME[i])
                        .on_hover_text("Click on the path in the right to edit the paths");
                    let links = ui
                        .add_enabled(*edit, Link::new(paths.to_string_lossy()))
                        .on_hover_text("Click to Edit");
                    if links.clicked() {
                        if let Some(p) = DynoFileManager::pick_folder("Change Path", &paths) {
                            *paths = p;
                        }
                    }
                    ui.end_row();
                }
            });
        ui.add_space(50.0);
    }
}
