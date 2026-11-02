use super::helpers::{run_event_loop_until_idle, test_config, test_key_sequence, AppBuilder};
use helix_term::{
    application::Application,
    compositor::{Component, Context, EventResult},
    job::Jobs,
    keymap::Keymaps,
    ui::EditorView,
};
use helix_view::{
    doc,
    editor::{Action, BufferLine},
    graphics::Rect,
    input::{self, parse_macro},
    view, DocumentId,
};
use tui::buffer::Buffer;

#[cfg(windows)]
use crossterm::event::{
    Event, KeyEvent, KeyModifiers as Modifiers, MouseButton, MouseEvent, MouseEventKind,
};
#[cfg(not(windows))]
use termina::event::{Event, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind};

async fn keys(app: &mut Application, input: &str) -> anyhow::Result<()> {
    for key in parse_macro(input)? {
        app.handle_terminal_events(Ok(Event::Key(KeyEvent::from(key))))
            .await;
    }
    Ok(())
}

async fn click(app: &mut Application, column: u16, row: u16) {
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        app.handle_terminal_events(Ok(Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: Modifiers::empty(),
        })))
        .await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn bufferline_click_boundaries_and_unicode() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut config = test_config();
    config.editor.bufferline = BufferLine::Always;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("aa"), None)
        .with_file(dir.path().join("界é"), None)
        .with_file(dir.path().join("ccc"), None)
        .build()?;
    let docs: Vec<_> = app.editor.documents().map(|doc| doc.id()).collect();
    keys(&mut app, "<esc>").await?;

    // Labels occupy [0, 4), [4, 9), and [9, 14) terminal columns.
    // Include padding, both cells of the wide character, and exact boundaries.
    for (column, expected) in [
        (9, docs[2]),
        (3, docs[0]),
        (4, docs[1]),
        (13, docs[2]),
        (5, docs[1]),
        (0, docs[0]),
        (6, docs[1]),
        (14, docs[1]),
        (119, docs[1]),
        (8, docs[1]),
    ] {
        click(&mut app, column, 0).await;
        assert_eq!(doc!(app.editor).id(), expected, "column {column}");
    }

    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn bufferline_modified_label_preserves_selections() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut config = test_config();
    config.editor.bufferline = BufferLine::Always;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("aa"), None)
        .with_file(dir.path().join("bb"), None)
        .with_input_text("#[α|]# #(β|)#\n")
        .build()?;
    let view_id = app.editor.tree.focus;
    let first = doc!(app.editor).id();
    let second = app.editor.documents().last().unwrap().id();
    let selection = doc!(app.editor).selection(view_id).clone();
    let text = doc!(app.editor).text().clone();
    keys(&mut app, "<esc>").await?;

    // The modification marker makes the first label " aa[+] ", seven cells wide.
    click(&mut app, 7, 0).await;
    assert_eq!(doc!(app.editor).id(), second);
    click(&mut app, 6, 0).await;
    assert_eq!(doc!(app.editor).id(), first);
    assert_eq!(doc!(app.editor).text(), &text);
    assert_eq!(doc!(app.editor).selection(view_id), &selection);

    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn bufferline_click_clipped_labels_after_resize() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut config = test_config();
    config.editor.bufferline = BufferLine::Always;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("aa"), None)
        .with_file(dir.path().join("bbbb"), None)
        .with_file(dir.path().join("ccc"), None)
        .build()?;
    let docs: Vec<_> = app.editor.documents().map(|doc| doc.id()).collect();

    // Render the component directly because Application's test backend has a fixed size.
    let mut editor_view = EditorView::new(Keymaps::default());
    let mut jobs = Jobs::new();
    let mut cx = Context {
        editor: &mut app.editor,
        jobs: &mut jobs,
        scroll: None,
    };
    for (width, column, expected) in [
        (7, 6, docs[1]),
        (4, 3, docs[0]),
        // The old hit region for the second label must be gone after shrinking.
        (4, 6, docs[0]),
        (120, 10, docs[2]),
    ] {
        let area = Rect::new(0, 0, width, 20);
        editor_view.render(area, &mut Buffer::empty(area), &mut cx);
        let event = input::Event::Mouse(input::MouseEvent {
            kind: input::MouseEventKind::Down(input::MouseButton::Left),
            column,
            row: 0,
            modifiers: input::KeyModifiers::empty(),
        });
        assert!(matches!(
            editor_view.handle_event(&event, &mut cx),
            EventResult::Consumed(_)
        ));
        assert_eq!(
            doc!(cx.editor).id(),
            expected,
            "width {width}, column {column}"
        );
    }

    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn bufferline_click_replaces_only_focused_split() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut config = test_config();
    config.editor.bufferline = BufferLine::Always;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("aa"), None)
        .with_file(dir.path().join("bb"), None)
        .build()?;
    let first = doc!(app.editor).id();
    let second = app.editor.documents().last().unwrap().id();
    let original_view = app.editor.tree.focus;
    app.editor.switch(first, Action::VerticalSplit);
    let focused_view = app.editor.tree.focus;
    keys(&mut app, "<esc>").await?;

    click(&mut app, 4, 0).await;
    assert_eq!(app.editor.tree.focus, focused_view);
    let views: Vec<_> = app.editor.tree.views().collect();
    assert_eq!(views.len(), 2);
    assert_eq!(view!(app.editor, original_view).doc, first);
    assert_eq!(view!(app.editor, focused_view).doc, second);

    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn bufferline_visibility_changes() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut config = test_config();
    config.editor.bufferline = BufferLine::Multiple;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(dir.path().join("aa"), None)
        .with_file(dir.path().join("bb"), None)
        .with_input_text("hello #[w|]#orld\n")
        .build()?;
    let docs: Vec<DocumentId> = app.editor.documents().map(|doc| doc.id()).collect();
    keys(&mut app, "<esc>").await?;
    click(&mut app, 7, 0).await;
    assert_eq!(doc!(app.editor).id(), docs[1]);

    keys(&mut app, ":set bufferline never<ret>").await?;
    run_event_loop_until_idle(&mut app).await;
    click(&mut app, 0, 0).await;
    assert_eq!(doc!(app.editor).id(), docs[1]);

    // With only one buffer, `multiple` must leave row zero available to text clicks.
    app.editor.switch(docs[0], Action::Replace);
    assert!(app.editor.close_document(docs[1], false).is_ok());
    keys(&mut app, ":set bufferline multiple<ret>").await?;
    run_event_loop_until_idle(&mut app).await;
    assert_eq!(app.editor.tree.views().next().unwrap().0.area.y, 0);
    let column = app
        .editor
        .tree
        .views()
        .next()
        .unwrap()
        .0
        .inner_area(doc!(app.editor))
        .x;
    click(&mut app, column, 0).await;
    assert_eq!(
        doc!(app.editor)
            .selection(app.editor.tree.focus)
            .primary()
            .cursor(doc!(app.editor).text().slice(..)),
        0
    );

    test_key_sequence(&mut app, Some(":qa!<ret>"), None, true).await
}
