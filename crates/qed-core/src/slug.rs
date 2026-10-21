//! URL slugs for pages and posts.

/// Turns a title into a URL slug: lowercase ASCII letters and digits,
/// separated by single dashes.
///
/// German umlauts and ß are spelled out (`ä` → `ae`); other letters lose
/// their accents or are transliterated (`é` → `e`, `Ж` → `zh`).
///
/// ```
/// assert_eq!(qed_core::slug::slugify("Über uns & Team"), "ueber-uns-team");
/// ```
pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());

    for c in title.chars() {
        let spelled_out = match c {
            'ä' | 'Ä' => "ae",
            'ö' | 'Ö' => "oe",
            'ü' | 'Ü' => "ue",
            'ß' => "ss",
            other => deunicode::deunicode_char(other).unwrap_or(""),
        };

        for ascii in spelled_out.chars() {
            if ascii.is_ascii_alphanumeric() {
                slug.push(ascii.to_ascii_lowercase());
            } else if !slug.is_empty() && !slug.ends_with('-') {
                slug.push('-');
            }
        }
    }

    // A title ending in punctuation leaves one dash behind.
    if slug.ends_with('-') {
        slug.pop();
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn lowercases_and_joins_words_with_dashes() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn spells_out_german_letters() {
        assert_eq!(slugify("Größe für Bären"), "groesse-fuer-baeren");
        assert_eq!(slugify("ÜBER"), "ueber");
    }

    #[test]
    fn transliterates_other_scripts_and_accents() {
        assert_eq!(slugify("Café Ženeva"), "cafe-zeneva");
        assert_eq!(slugify("Жук"), "zhuk");
    }

    #[test]
    fn collapses_punctuation_and_trims_dashes() {
        assert_eq!(slugify("  --Hello,   World!--  "), "hello-world");
    }

    #[test]
    fn keeps_digits() {
        assert_eq!(slugify("Episode 42: The Answer"), "episode-42-the-answer");
    }

    #[test]
    fn empty_or_symbol_only_titles_give_an_empty_slug() {
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("?!"), "");
    }
}
