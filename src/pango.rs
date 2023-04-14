use std::fmt::Write;

pub trait PangoEscapeExt {
    fn escape_pango_format<T: Write + Default>(self) -> T
    where
        Self: Sized;
}

impl<Iter: Iterator<Item = char>> PangoEscapeExt for Iter {
    fn escape_pango_format<T>(self) -> T
    where
        T: Write + Default,
    {
        let mut t = T::default();

        for c in self {
            let _ = match c {
                '>' => t.write_str("&gt;"),
                '<' => t.write_str("&lt;"),
                '&' => t.write_str("&amp;"),
                '\'' => t.write_str("&apos;"),
                '"' => t.write_str("&quot;"),
                _ => t.write_char(c),
            };
        }

        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_simple_string() {
        assert_eq!(
            "<span foreground=\"blue\" size=\"x-large\">Blue text</span>!"
                .chars()
                .escape_pango_format::<String>(),
            "&lt;span foreground=&quot;blue&quot; size=&quot;x-large&quot;&gt;Blue text&lt;/span&gt;!"
        );
    }
}
