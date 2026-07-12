//! Invite code generation module.
//!
//! Provides the `WORD_LIST` constant (200 Ukrainian common nouns) and the
//! `generate_unique_code` async function that picks two distinct words at random,
//! formats them as `"{word_a}-{word_b}"`, and retries on database collision.

/// 200 Ukrainian common nouns for invite code generation.
///
/// Selection criteria (all enforced):
/// - 4-9 Ukrainian Cyrillic characters, lowercase only
/// - Common nouns only — no verbs, adjectives, proper nouns, diminutives
/// - High-frequency vocabulary recognizable to any Ukrainian adult
/// - No politically charged, profane, religious, medical, brand, or military terms
/// - No `нова` or `пошта` (Nova Poshta brand collision)
/// - No body parts, no regional dialects, no abbreviations
pub const WORD_LIST: &[&str] = &[
    // Sky and weather (12)
    "сонце",
    "місяць",
    "зірка",
    "небо",
    "хмара",
    "вітер",
    "туман",
    "роса",
    "грім",
    "блиск",
    "веселка",
    "заграва",
    // Water (8)
    "вода",
    "річка",
    "море",
    "озеро",
    "став",
    "джерело",
    "водоспад",
    "болото",
    // Land (11)
    "гора",
    "поле",
    "долина",
    "берег",
    "острів",
    "пагорб",
    "скеля",
    "печера",
    "земля",
    "камінь",
    "яруга",
    // Trees and plants (14)
    "квітка",
    "дерево",
    "трава",
    "сосна",
    "береза",
    "верба",
    "тополя",
    "клен",
    "липа",
    "ялина",
    "лист",
    "гілка",
    "корінь",
    "кора",
    // Flowers (9)
    "сонях",
    "ромашка",
    "тюльпан",
    "бузок",
    "троянда",
    "фіалка",
    "нарцис",
    "конвалія",
    "барвінок",
    // Fruits, berries, vegetables (10)
    "яблуко",
    "груша",
    "слива",
    "вишня",
    "малина",
    "горіх",
    "гриб",
    "ягода",
    "морква",
    "гарбуз",
    // Mammals (13)
    "кінь",
    "вовк",
    "ведмідь",
    "лисиця",
    "заєць",
    "олень",
    "бджола",
    "їжак",
    "білка",
    "бобер",
    "лось",
    "кабан",
    "борсук",
    // Young animals (3)
    "цуценя",
    "кошеня",
    "пташеня",
    // Birds (5)
    "птах",
    "орел",
    "ластівка",
    "горобець",
    "зозуля",
    // Animal features (3)
    "крило",
    "перо",
    "хвіст",
    // Household objects (15)
    "стіл",
    "вікно",
    "двері",
    "килим",
    "чашка",
    "ложка",
    "горщик",
    "кошик",
    "торба",
    "ключ",
    "замок",
    "скриня",
    "ліхтар",
    "люлька",
    "миска",
    // Clothing and textiles (10)
    "сорочка",
    "шапка",
    "рукав",
    "кишеня",
    "гудзик",
    "нитка",
    "голка",
    "тканина",
    "шарф",
    "пояс",
    // Tools and stationery (5)
    "ножиці",
    "гачок",
    "олівець",
    "папір",
    "зошит",
    // Precious things (7)
    "скарб",
    "монета",
    "перлина",
    "кристал",
    "бурштин",
    "золото",
    "срібло",
    // Music instruments (9)
    "барабан",
    "флейта",
    "скрипка",
    "гітара",
    "бандура",
    "дзвін",
    "сопілка",
    "цимбали",
    "кобза",
    // Abstract concepts (16)
    "мрія",
    "доля",
    "честь",
    "правда",
    "воля",
    "надія",
    "радість",
    "спокій",
    "віра",
    "думка",
    "слово",
    "голос",
    "серце",
    "шлях",
    "світло",
    "краса",
    // Art and culture (5)
    "казка",
    "пісня",
    "музика",
    "танець",
    "картина",
    // Time (12)
    "година",
    "хвилина",
    "ранок",
    "вечір",
    "день",
    "тиждень",
    "зима",
    "весна",
    "літо",
    "осінь",
    "світанок",
    "сутінки",
    // Transport (7)
    "літак",
    "потяг",
    "корабель",
    "велосипед",
    "човен",
    "сани",
    "ракета",
    // Places and architecture (10)
    "дорога",
    "стежка",
    "міст",
    "ворота",
    "паркан",
    "стіна",
    "підлога",
    "сходи",
    "кімната",
    "садиба",
    // People (4)
    "юнак",
    "дівчина",
    "друг",
    "дитина",
    // Fire and light (4)
    "вогонь",
    "сяйво",
    "іскра",
    "промінь",
    // Food (3)
    "хліб",
    "масло",
    "каша",
    // Landscape and direction (5)
    "повітря",
    "обрій",
    "захід",
    "північ",
    "рівнина",
];

/// Generates two-word invite code format string from two distinct word list entries.
///
/// This is the pure (no-DB) half of code generation; exposed for unit testing.
///
/// # Panics
///
/// Panics if `WORD_LIST` has fewer than 2 entries (compile-time guarantee violated).
#[cfg(any(feature = "ssr", test))]
#[must_use]
pub fn pick_two_words(rng: &mut impl rand::Rng) -> (&'static str, &'static str) {
    use rand::seq::IndexedRandom as _;
    let mut iter = WORD_LIST.choose_multiple(rng, 2);
    let a = iter.next().expect("WORD_LIST has >= 2 entries");
    let b = iter.next().expect("WORD_LIST has >= 2 entries");
    (a, b)
}

/// Format two words as a hyphenated invite code.
#[must_use]
pub fn format_code(word_a: &str, word_b: &str) -> String {
    format!("{word_a}-{word_b}")
}

/// Generates a unique invite code, retrying on collision up to 20 times.
///
/// Uses `WORD_LIST` to pick two distinct words, formats as `"{word_a}-{word_b}"`,
/// and checks the `invite_codes` table for existing codes with the same value.
///
/// # Errors
///
/// Returns `Err(sqlx::Error::RowNotFound)` after 20 failed collision checks —
/// astronomically unlikely with 200 words (~40 000 combinations). Any earlier
/// error is a genuine database failure forwarded from sqlx.
#[cfg(feature = "ssr")]
pub async fn generate_unique_code(pool: &sqlx::PgPool) -> Result<String, sqlx::Error> {
    for _ in 0..20 {
        // Generate the candidate inside a block so `rng` (which is `!Send`)
        // is dropped before the `.await` below. Holding `ThreadRng` across an
        // await point would violate the `Send` bound required by Leptos server
        // functions.
        let candidate = {
            let mut rng = rand::rng();
            let (a, b) = pick_two_words(&mut rng);
            format_code(a, b)
        };

        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM invite_codes WHERE code = $1) AS "exists!""#,
            candidate,
        )
        .fetch_one(pool)
        .await?;

        if !exists {
            return Ok(candidate);
        }
    }

    // 20 consecutive collisions from a ~40 000-combination space is
    // astronomically unlikely under normal operation — treat as fatal.
    Err(sqlx::Error::RowNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::collections::HashSet;

    /// Verify that `WORD_LIST` has exactly 200 entries.
    #[test]
    fn test_word_list_length() {
        assert_eq!(
            WORD_LIST.len(),
            200,
            "WORD_LIST must have exactly 200 entries"
        );
    }

    /// Verify that all entries are unique (no duplicates).
    #[test]
    fn test_word_list_no_duplicates() {
        let unique: HashSet<&&str> = WORD_LIST.iter().collect();
        assert_eq!(
            unique.len(),
            WORD_LIST.len(),
            "WORD_LIST contains duplicate entries"
        );
    }

    /// Verify that all entries are 4-9 lowercase Cyrillic characters.
    #[test]
    fn test_word_list_format() {
        for word in WORD_LIST {
            let char_count = word.chars().count();
            assert!(
                (4..=9).contains(&char_count),
                "word {word:?} has {char_count} chars (must be 4-9)"
            );
            for ch in word.chars() {
                assert!(
                    ch.is_alphabetic() && !ch.is_ascii(),
                    "word {word:?} contains non-Cyrillic character {ch:?}"
                );
                assert!(
                    ch.is_lowercase(),
                    "word {word:?} contains uppercase character {ch:?}"
                );
            }
        }
    }

    /// Verify the code format: two Cyrillic word segments joined by a hyphen.
    #[test]
    fn test_generated_code_format() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let (a, b) = pick_two_words(&mut rng);
            let code = format_code(a, b);
            let parts: Vec<&str> = code.splitn(2, '-').collect();
            assert_eq!(
                parts.len(),
                2,
                "code {code:?} must split into 2 parts on '-'"
            );
            for part in &parts {
                assert!(!part.is_empty(), "code {code:?} has empty part");
                for ch in part.chars() {
                    assert!(
                        ch.is_alphabetic() && !ch.is_ascii(),
                        "code {code:?} part {part:?} contains non-Cyrillic char {ch:?}"
                    );
                }
            }
        }
    }

    /// Verify that both words in each generated code come from `WORD_LIST`.
    #[test]
    fn test_words_come_from_word_list() {
        let word_set: HashSet<&&str> = WORD_LIST.iter().collect();
        let mut rng = rand::rng();
        for _ in 0..200 {
            let (a, b) = pick_two_words(&mut rng);
            assert!(word_set.contains(&a), "word {a:?} is not in WORD_LIST");
            assert!(word_set.contains(&b), "word {b:?} is not in WORD_LIST");
        }
    }

    /// Verify that the two chosen words are always distinct.
    #[test]
    fn test_two_words_are_distinct() {
        let mut rng = rand::rng();
        for _ in 0..500 {
            let (a, b) = pick_two_words(&mut rng);
            assert_ne!(a, b, "pick_two_words returned the same word twice: {a:?}");
        }
    }

    /// Verify that 100 generated codes are all unique (no repeats in a single run).
    ///
    /// Uses the pure `pick_two_words` path — no database needed.
    /// With 200 words and 200×199 = 39 800 ordered combinations, a collision
    /// in 100 draws indicates a bug (birthday problem: p ≈ 0.12%).
    #[test]
    fn test_100_generated_codes_are_unique() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mut seen: HashSet<String> = HashSet::new();
        for _ in 0..100 {
            let (a, b) = pick_two_words(&mut rng);
            let code = format_code(a, b);
            assert!(
                seen.insert(code.clone()),
                "duplicate code generated: {code:?}"
            );
        }
    }
}
