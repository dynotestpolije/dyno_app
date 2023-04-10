use dyno_types::reqwest::Url;
use eframe::egui::Hyperlink;

static FONT_ICONS: &'_ [(&'_ str, char)] = &[
    // Warnings
    ("ftp", '\u{26A0}'),
    ("http", '\u{26A0}'),
    ("https", '\u{1F30D}'),
    ("telnet", '\u{26A0}'),
    // URI schemes
    ("appstream", '\u{1F4E6}'),
    ("apt", '\u{1F4E6}'),
    ("fax", '\u{1F4E0}'),
    ("fb", '\u{E604}'),
    ("file", '\u{1F5C1}'),
    ("flatpak", '\u{1F4E6}'),
    ("gemini", '\u{264A}'),
    ("geo", '\u{1F5FA}'),
    ("git", '\u{E625}'),
    ("info", '\u{2139}'),
    ("ipp", '\u{1F5B6}'),
    ("ipps", '\u{1F5B6}'),
    ("irc", '\u{1F4AC}'),
    ("irc6", '\u{1F4AC}'),
    ("ircs", '\u{1F4AC}'),
    ("itms-apps", '\u{F8FF}'),
    ("ldap", '\u{1F4D5}'),
    ("ldaps", '\u{1F4D5}'),
    ("mailto", '\u{1F4E7}'),
    ("maps", '\u{1F5FA}'),
    ("market", '\u{E618}'),
    ("message", '\u{1F4E7}'),
    ("ms-", '\u{E61F}'), // Not a typo.
    ("nfs", '\u{1F5C1}'),
    ("pkg", '\u{1F4E6}'),
    ("rpm", '\u{1F4E6}'),
    ("sftp", '\u{1F5C1}'),
    ("sip", '\u{1F4DE}'),
    ("sips", '\u{1F4DE}'),
    ("skype", '\u{E613}'),
    ("smb", '\u{1F5C1}'),
    ("sms", '\u{2709}'),
    ("snap", '\u{1F4E6}'),
    ("ssh", '\u{1F5A5}'),
    ("steam", '\u{E623}'),
    ("tel", '\u{1F4DE}'),
    // Websites
    ("https://apps.apple.com/", '\u{F8FF}'),
    ("https://crates.io/", '\u{1F4E6}'),
    ("https://docs.rs/", '\u{1F4DA}'),
    ("https://drive.google.com/", '\u{E62F}'),
    ("https://play.google.com/store/apps/", '\u{E618}'),
    ("https://soundcloud.com/", '\u{E627}'),
    ("https://stackoverflow.com/", '\u{E601}'),
    ("https://steamcommunity.com/", '\u{E623}'),
    ("https://store.steampowered.com/", '\u{E623}'),
    ("https://twitter.com/", '\u{E603}'),
    ("https://vimeo.com/", '\u{E602}'),
    ("https://www.dropbox.com/", '\u{E610}'),
    ("https://www.facebook.com/", '\u{E604}'),
    ("https://www.instagram.com/", '\u{E60F}'),
    ("https://www.paypal.com/", '\u{E616}'),
    ("https://www.youtube.com/", '\u{E636}'),
    ("https://youtu.be/", '\u{E636}'),
    // Generic git rules
    ("https://git.com/", '\u{E625}'),
    ("https://cgit.com/", '\u{E625}'),
    ("https://gitlab.com/", '\u{E625}'),
    ("https://RizalAchp.github.io/", '\u{E624}'),
    ("https://RizalAchp.gitlab.io/", '\u{E625}'),
    ("https://RizalAchp.reddit.com/", '\u{E628}'),
    // Non-exhaustive list of some git instances not covered by the generic rules
    ("https://bitbucket.org/", '\u{E625}'),
    ("https://code.qt.io/", '\u{E625}'),
    ("https://code.videolan.org/", '\u{E625}'),
    ("https://framagit.org/", '\u{E625}'),
    ("https://gitee.com/", '\u{E625}'),
    ("https://github.com/", '\u{E624}'),
    ("https://invent.kde.org/", '\u{E625}'),
    ("https://salsa.debian.org/", '\u{E625}'),
    // Discord and friends have no symbols in the default emoji font.
];

#[rustfmt::skip]
fn hyperlink_icon(url: impl AsRef<str>) -> char {
    let Ok(url) = Url::parse(url.as_ref()) else {
        return '\u{2BA9}';
    };
    let scheme: &'_ str = url.scheme();
    if let Some((_, icon)) = FONT_ICONS[0..40].iter().find(|(key, _)| key.eq(&scheme)) {
        return *icon;
    }
    if let Some(host) = url.host_str() {
        let base = format!("{}://{}/", scheme, host);
        return FONT_ICONS[40..].iter().find_map(|(key, icon)| if key.eq(&base) { Some(*icon) } else { None }).unwrap_or('\u{2BA9}');
    }

    '\u{2BA9}'
}

pub fn hyperlink_with_icon(url: impl AsRef<str>) -> Hyperlink {
    Hyperlink::from_label_and_url(
        format!(
            "{icon} {urlstr}",
            icon = hyperlink_icon(url.as_ref()),
            urlstr = url.as_ref()
        ),
        url.as_ref(),
    )
}

pub fn hyperlink_with_icon_to(label: impl AsRef<str>, url: impl AsRef<str>) -> Hyperlink {
    Hyperlink::from_label_and_url(
        format!("{} {}", hyperlink_icon(url.as_ref()), label.as_ref()),
        url.as_ref(),
    )
}
