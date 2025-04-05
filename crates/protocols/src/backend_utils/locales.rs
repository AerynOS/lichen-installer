// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::Deref;

use crate::lichen;

impl<'a> From<locales_rs::Locale<'a>> for lichen::locales::Locale {
    fn from(value: locales_rs::Locale<'a>) -> Self {
        lichen::locales::Locale {
            name: value.name.clone(),
            display_name: value.display_name.clone(),
            language: Some(value.language.into()),
            territory: Some(value.territory.into()),
            modifier: value.modifier.clone(),
            codeset: value.codeset.clone(),
        }
    }
}

impl<T> From<T> for lichen::locales::Language
where
    T: Deref<Target = locales_rs::Language>,
{
    fn from(value: T) -> Self {
        lichen::locales::Language {
            code: value.code.clone(),
            code2: value.code2.clone(),
            display_name: value.display_name.clone(),
            inverted_name: value.inverted_name.clone(),
        }
    }
}

impl<T> From<T> for lichen::locales::Territory
where
    T: Deref<Target = locales_rs::Territory>,
{
    fn from(value: T) -> Self {
        lichen::locales::Territory {
            code: value.code.clone(),
            code2: value.code2.clone(),
            display_name: value.display_name.clone(),
            flag: value.flag.clone(),
        }
    }
}
