use super::*;
use helix_core::doc_formatter::{DocumentFormatter, TextFormat};
use helix_term::application::Application;
use helix_view::{current_ref, input::parse_macro};

#[cfg(windows)]
use crossterm::event::{Event, KeyEvent};
#[cfg(not(windows))]
use termina::event::{Event, KeyEvent};

async fn keys(app: &mut Application, input: &str) -> anyhow::Result<()> {
    for key in parse_macro(input)? {
        app.handle_terminal_events(Ok(Event::Key(KeyEvent::from(key))))
            .await;
    }
    Ok(())
}

fn displayed_words(app: &Application) -> Vec<String> {
    let (view, doc) = current_ref!(app.editor);
    let annotations = view.text_annotations(doc, None);
    let format = TextFormat::default();
    DocumentFormatter::new_at_prev_checkpoint(doc.text().slice(..), &format, &annotations, 0)
        .filter(|grapheme| !grapheme.source.is_eof())
        .map(|grapheme| grapheme.raw.to_string())
        .collect::<String>()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn jump_labels_prefer_earlier_characters_and_reach_every_candidate() -> anyhow::Result<()> {
    // Cover each row, column, and corner through the entire four-character
    // alphabet. The seventeenth word must remain unlabeled at capacity.
    let labels = [
        "aa", "ba", "ab", "bb", "ca", "cb", "ac", "bc", "cc", "da", "db", "dc", "ad", "bd", "cd",
        "dd",
    ];
    for alphabet in ["abcd", "sdfj", "αβγδ"] {
        let alphabet: Vec<_> = alphabet.chars().collect();
        let expected: Vec<String> = labels
            .iter()
            .map(|label| {
                label
                    .bytes()
                    .map(|ch| alphabet[(ch - b'a') as usize])
                    .collect()
            })
            .collect();
        let mut config = test_config();
        config.editor.jump_label_alphabet = alphabet;
        let mut app = AppBuilder::new()
            .with_config(config)
            .with_input_text(format!("#[ |]#{}\n", ["xx"; 17].join(" ")))
            .build()?;

        for (i, label) in expected.iter().enumerate() {
            keys(&mut app, "gggw").await?;
            let mut displayed = expected.clone();
            displayed.push("xx".into());
            assert_eq!(displayed_words(&app), displayed);

            keys(&mut app, label).await?;
            let (view, doc) = current_ref!(app.editor);
            assert_eq!(
                doc.selection(view.id),
                &Selection::single(1 + 3 * i, 3 + 3 * i),
                "label {label}"
            );
            assert_eq!(displayed_words(&app), vec!["xx"; 17]);
        }
        test_key_sequence(&mut app, Some(":q!<ret>"), None, true).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn jump_labels_partial_square_and_cancellation() -> anyhow::Result<()> {
    let mut app = AppBuilder::new()
        .with_input_text("#[ |]#xx xx xx xx xx\n")
        .build()?;
    // The final square is incomplete: ca exists, but cb and cc do not.
    for label in [
        "cb", "cc", "zz", "!", "a!", "<esc>", "a<esc>", "<C-a>", "a<C-a>",
    ] {
        keys(&mut app, "gw").await?;
        assert_eq!(displayed_words(&app), ["aa", "ba", "ab", "bb", "ca"]);
        keys(&mut app, label).await?;
        let (view, doc) = current_ref!(app.editor);
        assert_eq!(doc.selection(view.id), &Selection::single(0, 1));
        assert_eq!(displayed_words(&app), vec!["xx"; 5]);
    }
    keys(&mut app, "gwca").await?;
    let (view, doc) = current_ref!(app.editor);
    assert_eq!(doc.selection(view.id), &Selection::single(13, 15));
    test_key_sequence(&mut app, Some(":q!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn jump_labels_unicode_graphemes_and_multiple_selections() -> anyhow::Result<()> {
    let mut app = AppBuilder::new()
        .with_input_text("#[ |]#e\u{301}té #(界|)#面 αβ\n")
        .build()?;
    keys(&mut app, "gw").await?;
    // Keep combining marks intact and align labels on later multibyte words.
    assert_eq!(displayed_words(&app), ["e\u{301}aa", "ba", "ab"]);
    keys(&mut app, "ba").await?;
    let (view, doc) = current_ref!(app.editor);
    assert_eq!(doc.selection(view.id), &Selection::single(6, 8));
    assert_eq!(doc.text().to_string(), " e\u{301}té 界面 αβ\n");
    assert_eq!(displayed_words(&app), ["e\u{301}té", "界面", "αβ"]);
    test_key_sequence(&mut app, Some(":q!<ret>"), None, true).await
}

#[tokio::test(flavor = "multi_thread")]
async fn jump_labels_backward_and_extend() -> anyhow::Result<()> {
    // Candidates alternate forward/backward from the primary cursor.
    test((
        "one two #[ |]#three four",
        "gwba",
        "one #[two|]#  three four",
    ))
    .await?;
    test((
        "one two #[ |]#three four",
        "gwbb",
        "#[one|]# two  three four",
    ))
    .await?;
    test((
        "one two #[ |]#three four",
        "vgwab",
        "one two #[ three four|]#",
    ))
    .await?;
    test((
        "one two #[ |]#three four",
        "vgwbb",
        "#[|one two  ]#three four",
    ))
    .await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn jump_labels_empty_and_single_character_alphabets() -> anyhow::Result<()> {
    for (alphabet, input) in [("", "gw"), ("a", "gwaa")] {
        let mut config = test_config();
        config.editor.jump_label_alphabet = alphabet.chars().collect();
        test_with_config(
            AppBuilder::new().with_config(config),
            (
                "#[ |]#xx yy",
                input,
                if alphabet.is_empty() {
                    "#[ |]#xx yy"
                } else {
                    " #[xx|]# yy"
                },
            ),
        )
        .await?;
    }
    test(("#[ |]#", "gw", "#[ |]#")).await?;
    Ok(())
}
