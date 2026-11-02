use super::helpers::{test_config, test_key_sequence, AppBuilder};
use helix_term::{
    compositor::{Component, Context},
    config::{Config, ConfigLoadError},
    job::Jobs,
    keymap::Keymaps,
    ui::EditorView,
};
use helix_view::{
    doc,
    editor::{BufferLine, StatusLineElement},
    graphics::{Color, Rect},
    icons::{Icons, ICONS},
    input, view,
};
use std::sync::Arc;
use tui::buffer::Buffer;

#[tokio::test(flavor = "multi_thread")]
async fn icons_config_merge_reload_and_invalid_config() -> anyhow::Result<()> {
    let global = r##"
        [icons.fs]
        enable = true
        [icons.fs.file.extension]
        rs = { glyph = "R", fg = "#ff0000" }
        custom = "C"
        [icons.ui.statusline]
        separator = "!"
    "##
    .to_owned();
    let local = r#"
        [icons.fs.file.extension]
        rs = { glyph = "界" }
        [icons.diagnostic]
        error = "E"
    "#
    .to_owned();
    Config::load(Ok(&global), Ok(local)).unwrap();
    {
        let icons = ICONS.load();
        let file = icons.fs().file().unwrap();
        let rust = file.get("main.rs").unwrap();
        assert_eq!(rust.with_no_padding().glyph(), "界");
        assert_eq!(rust.style().fg, Some(Color::Rgb(255, 0, 0)));
        assert_eq!(file.get("file.custom").unwrap().glyph(), "C");
        assert_eq!(icons.ui().statusline().separator().glyph(), "!");
        assert_eq!(icons.diagnostic().error().glyph(), "E");
    }

    // Failed reloads must leave the last valid icons in place.
    for invalid in [
        "[icons]\nunknown = true",
        "[icons.diagnostic]\nerror = { fg = 'invalid' }",
        "[editor]\nunknown = true\n[icons.ui.statusline]\nseparator = '?'",
    ] {
        assert!(Config::load(Ok(&invalid.to_owned()), Err(ConfigLoadError::default())).is_err());
        assert_eq!(ICONS.load().ui().statusline().separator().glyph(), "!");
    }

    Config::load(Ok(&String::new()), Err(ConfigLoadError::default())).unwrap();
    assert!(ICONS.load().fs().file().is_none());
    assert_eq!(ICONS.load().ui().statusline().separator().glyph(), "│");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn icons_bufferline_clips_padded_wide_icons() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut config = test_config();
    config.editor.bufferline = BufferLine::Always;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("one.custom"), None)
        .with_file(dir.path().join("two.custom"), None)
        .build()?;
    ICONS.store(Arc::new(toml::from_str::<Icons>(
        r#"
        [fs]
        enable = true
        [fs.file.extension]
        custom = { glyph = "界", padding = { left = 1, right = 1 } }
    "#,
    )?));
    let mut editor_view = EditorView::new(Keymaps::default());
    let area = Rect::new(2, 0, 8, 1);
    let mut surface = Buffer::empty(Rect::new(0, 0, 16, 1));
    surface.set_string(10, 0, "guard!", Default::default());
    editor_view.render_bufferline(&app.editor, area, &mut surface);
    assert_eq!(surface[(3, 0)].symbol, "界");
    assert_eq!(surface[(6, 0)].symbol, "o");
    assert_eq!(surface[(10, 0)].symbol, "g");

    let docs: Vec<_> = app.editor.documents().map(|doc| doc.id()).collect();
    let area = Rect::new(0, 0, 40, 1);
    editor_view.render_bufferline(&app.editor, area, &mut Buffer::empty(area));
    // Four columns of icon/padding, ten of filename, and a trailing space.
    // The icon, both cells of the wide glyph, and label padding are clickable.
    for (column, expected) in [
        (15, docs[1]),
        (1, docs[0]),
        (16, docs[1]),
        (2, docs[0]),
        (14, docs[0]),
    ] {
        editor_view.handle_event(
            &input::Event::Mouse(input::MouseEvent {
                kind: input::MouseEventKind::Down(input::MouseButton::Left),
                column,
                row: 0,
                modifiers: input::KeyModifiers::empty(),
            }),
            &mut Context {
                editor: &mut app.editor,
                jobs: &mut Jobs::new(),
                scroll: None,
            },
        );
        assert_eq!(doc!(app.editor).id(), expected, "column {column}");
    }
    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn icons_scratch_buffer_keeps_statusline_file_type() -> anyhow::Result<()> {
    let mut config = test_config();
    config.editor.statusline.left = vec![StatusLineElement::FileType];
    config.editor.statusline.right.clear();
    let mut app = AppBuilder::new().with_config(config).build()?;
    ICONS.store(Arc::new(toml::from_str::<Icons>("[fs]\nenable = true")?));
    let area = Rect::new(0, 0, 80, 10);
    let mut surface = Buffer::empty(area);
    EditorView::new(Keymaps::default()).render(
        area,
        &mut surface,
        &mut Context {
            editor: &mut app.editor,
            jobs: &mut Jobs::new(),
            scroll: None,
        },
    );
    let statusline_y = view!(app.editor).area.bottom() - 1;
    let statusline: String = (0..80)
        .map(|x| surface[(x, statusline_y)].symbol.to_string())
        .collect();
    assert!(statusline.contains("text"), "{statusline:?}");
    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn icons_large_tab_width_and_virtual_glyphs() -> anyhow::Result<()> {
    let mut config = test_config();
    config.editor.whitespace.render =
        helix_view::editor::WhitespaceRender::Basic(helix_view::editor::WhitespaceRenderValue::All);
    config.editor.indent_guides.render = true;
    let dir = tempfile::tempdir()?;
    std::fs::write(
        dir.path().join(".editorconfig"),
        "root = true\n[*]\ntab_width = 32\nindent_style = tab\n",
    )?;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("input.custom"), None)
        .with_input_text("\t\t#[α|]#\n")
        .build()?;
    assert_eq!(doc!(app.editor).tab_width(), 32);
    ICONS.store(Arc::new(toml::from_str::<Icons>(
        r#"
        [ui.virtual]
        tab = ">"
        tabpad = "."
        indentation = "!"
        wrap = "W "
    "#,
    )?));
    let area = Rect::new(0, 0, 80, 10);
    let mut surface = Buffer::empty(area);
    let mut editor_view = EditorView::new(Keymaps::default());
    editor_view.render(
        area,
        &mut surface,
        &mut Context {
            editor: &mut app.editor,
            jobs: &mut Jobs::new(),
            scroll: None,
        },
    );
    let inner = view!(app.editor).inner_area(doc!(app.editor));
    assert_eq!(surface[(inner.x, inner.y)].symbol, "!");
    assert_eq!(surface[(inner.x + 32, inner.y)].symbol, "!");
    assert_eq!(doc!(app.editor).text_format(80, None).wrap_indicator, "W ");
    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn icons_ruler_preserves_text_and_marks_empty_columns() -> anyhow::Result<()> {
    let mut config = test_config();
    config.editor.rulers = vec![3];
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_input_text("ab#[c|]#def\n")
        .build()?;
    let area = Rect::new(0, 0, 80, 10);
    let mut surface = Buffer::empty(area);
    let mut editor_view = EditorView::new(Keymaps::default());
    for (config, expected) in [("", " "), ("[ui.virtual]\nruler = '|'", "|")] {
        ICONS.store(Arc::new(toml::from_str::<Icons>(config)?));
        editor_view.render(
            area,
            &mut surface,
            &mut Context {
                editor: &mut app.editor,
                jobs: &mut Jobs::new(),
                scroll: None,
            },
        );
        let inner = view!(app.editor).inner_area(doc!(app.editor));
        assert_eq!(surface[(inner.x + 2, inner.y)].symbol, "c");
        assert_eq!(surface[(inner.x + 3, inner.y)].symbol, "d");
        assert_eq!(surface[(inner.x + 2, inner.y + 1)].symbol, expected);
    }
    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}
