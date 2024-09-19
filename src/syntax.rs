use similar::ChangeTag;
use subslice_offset::SubsliceOffset;
use syntect::{
    easy::HighlightLines,
    highlighting::{Color, FontStyle, Style, Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

lazy_static::lazy_static! {
    /// The default syntect syntax set, used for parsing language definitions.
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    /// The default syntect theme set, currently only one theme is ever used.
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
    /// Theme definition from the default syntect theme set.
    static ref THEME: &'static Theme = &THEME_SET.themes["InspiredGitHub"];
}

/// Get the `SyntaxReference` from `SYNTAX_SET` to use for syntax highlighting
/// the given file.
///
/// It will be based first on the file's name, then the file's extension, and
/// finally based on the first line of the file.
pub fn infer_syntax_for_file(path: &str, first_line: Option<&str>) -> &'static SyntaxReference {
    // Determine which syntax should be used for this file. It will be based
    // first on the file's name, then the file's extension, then the first line.
    let (_, file_name) = path.rsplit_once('/').unwrap_or(("", path));
    let (_, extension) = file_name.rsplit_once('.').unwrap_or(("", file_name));
    SYNTAX_SET
        .find_syntax_by_extension(file_name)
        .or_else(|| SYNTAX_SET.find_syntax_by_extension(extension))
        .or_else(|| first_line.and_then(|line| SYNTAX_SET.find_syntax_by_first_line(line)))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
}

/// Highlight a single line as a bytes slice, avoiding extra copies.
fn highlight_bytes_line(
    highlight_lines: &mut HighlightLines<'_>,
    tag: ChangeTag,
    bytes: &bytes::Bytes,
) -> Option<Vec<(Style, bytes::Bytes)>> {
    // Don't highlight removal lines, as it could confuse the parser.
    if tag == ChangeTag::Delete {
        return None;
    }

    let line = std::str::from_utf8(&bytes[..]).ok()?;
    let styles = highlight_lines.highlight_line(line, &SYNTAX_SET).ok()?;

    // Map each chunk back to the bytes slice to avoid unnecessary copies.
    Some(
        styles
            .into_iter()
            .map(|(style, chunk)| {
                let start = line[..].subslice_offset(chunk).unwrap();
                (style, bytes.slice(start..start + chunk.len()))
            })
            .collect::<Vec<_>>(),
    )
}

/// Apply syntax highlighting to a list of changes using the listed syntax.
pub fn highlight_changes(
    syntax: &'static SyntaxReference,
    changes: &[(ChangeTag, bytes::Bytes)],
) -> Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)> {
    let default_style = Style {
        foreground: THEME.settings.foreground.unwrap_or(Color::BLACK),
        background: THEME.settings.background.unwrap_or(Color::WHITE),
        font_style: FontStyle::empty(),
    };

    let mut highlight_lines = HighlightLines::new(syntax, &THEME);
    changes
        .iter()
        .map(|(tag, bytes)| {
            let styled = highlight_bytes_line(&mut highlight_lines, *tag, bytes)
                .unwrap_or_else(|| vec![(default_style, bytes.clone())]);
            (*tag, styled)
        })
        .collect()
}

/// Convert the given syntect style to inline `style` attribute formatting.
///
/// Does not apply background colors.
pub fn syntect_style_to_css(style: &Style) -> String {
    let mut css = format!(
        "color:#{:02x}{:02x}{:02x};",
        style.foreground.r, style.foreground.g, style.foreground.b
    );
    if style.font_style.contains(FontStyle::UNDERLINE) {
        css.push_str("text-decoration:underline;");
    }
    if style.font_style.contains(FontStyle::BOLD) {
        css.push_str("font-weight:bold;");
    }
    if style.font_style.contains(FontStyle::BOLD) {
        css.push_str("font-style:italic;");
    }
    css
}
