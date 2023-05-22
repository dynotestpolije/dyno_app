#![allow(unused)]

use dyno_core::paste::paste;
use std::path::{Path, PathBuf};

pub type Filters = &'static [(&'static str, &'static [&'static str])];

pub struct DynoFileManager;

impl DynoFileManager {
    #[inline(always)]
    pub fn pick_folders<P: AsRef<Path>>(title: &'_ str, dir: P) -> Option<Vec<PathBuf>> {
        rfd::FileDialog::new()
            .set_directory(dir)
            .set_title(title)
            .pick_folders()
    }

    #[inline(always)]
    pub fn pick_folder<P: AsRef<Path>>(title: &'_ str, dir: P) -> Option<PathBuf> {
        Self::pick_folders(title, dir).map(|x| x.first().cloned())?
    }

    pub fn pick_files<P: AsRef<Path>>(
        title: &'_ str,
        dir: P,
        filters: Filters,
    ) -> Option<Vec<PathBuf>> {
        let mut file = rfd::FileDialog::new().set_directory(dir).set_title(title);
        for (name, ext) in filters {
            file = file.add_filter(name, ext);
        }
        file.pick_files()
    }

    #[inline(always)]
    pub fn pick_file<P: AsRef<Path>>(title: &'_ str, dir: P, filters: Filters) -> Option<PathBuf> {
        Self::pick_files(title, dir, filters).map(|x| x.first().cloned())?
    }

    #[inline]
    pub fn save_file<P: AsRef<Path>, S: AsRef<str>>(
        title: &'_ str,
        file: S,
        dir: P,
        filters: Filters,
    ) -> Option<PathBuf> {
        let mut file = rfd::FileDialog::new()
            .set_title("Save File Dynotest")
            .set_directory(dir.as_ref())
            .set_file_name(file.as_ref());
        for (name, ext) in filters {
            file = file.add_filter(name, ext);
        }
        file.save_file()
    }
}

macro_rules! impl_file_picker {
    ( $( $name:ident -> [$($tuples:tt)*] ),* $(,)?) => {
        impl DynoFileManager {
            paste!($(
                #[allow(unused)]
                #[inline(always)]
                pub fn [<pick_ $name>]<P>(dir: P) -> Option<PathBuf>
                where
                    P: AsRef<Path>,
                {
                    Self::pick_file(
                        concat!("Pick `", stringify!([<$name:camel>]), "` File Dynotest"),
                        dir, &[$($tuples)*]
                    )
                }
                #[allow(unused)]
                #[inline(always)]
                pub fn [<save_ $name>]<P, S>(file: S, dir: P) -> Option<PathBuf>
                    where P: AsRef<Path>, S: AsRef<str>
                {
                    Self::save_file(
                        concat!("Save `", stringify!([<$name:camel>]), "` File Dynotest"),
                        file, dir, &[$($tuples)*]
                    )
                }
            )*);
        }
    };
}

impl_file_picker!(
    all_type    -> [
        ("Dyno Binaries File",  &["dbin", "dynobin"]),
        ("Binaries File",       &["bin"]),
        ("Csv File",            &["csv", "dcsv"]),
        ("Excel File",          &["xlsx", "xls"]),
    ],
    binaries    -> [("Dyno Binaries File", &["dbin", "dynobin"]), ("Binaries File", &["bin"])],
    csv         -> [("Csv File",      &["csv", "dcsv"])],
    excel       -> [("Excel File",    &["xlsx", "xls"])]
);
