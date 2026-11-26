//! Trait untuk enum pilihan tetap pada parameter slash command (setara `choice_parameter.rs` di poise).

use std::borrow::Cow;

/// Informasi satu pilihan pada [`ChoiceParameter`].
#[derive(Debug, Clone)]
pub struct ChoiceParameterInfo {
    /// Nama yang ditampilkan Discord kepada pengguna.
    pub name: Cow<'static, str>,
    /// Nilai yang dikirim balik ke bot saat dipilih (sama dengan `name` bila tidak diatur).
    pub value: Cow<'static, str>,
    /// Daftar lokalisasi `(locale, localized_name)`.
    pub localizations: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

/// Trait untuk tipe enum yang bisa dipakai sebagai pilihan tetap (choices) di slash command.
pub trait ChoiceParameter: Sized {
    /// Daftar semua varian pilihan yang tersedia.
    fn list() -> Vec<ChoiceParameterInfo>;

    /// Ambil varian berdasarkan index di [`Self::list()`].
    fn from_index(index: usize) -> Option<Self>;

    /// Parse varian dari nama (atau nama terlokalisasi).
    fn from_name(name: &str) -> Option<Self>;

    /// Nama kanonik dari varian ini.
    fn name(&self) -> &'static str;

    /// Nama terlokalisasi untuk `locale` tertentu, kalau ada.
    fn localized_name(&self, locale: &str) -> Option<&'static str> {
        let _ = locale;
        None
    }
}
