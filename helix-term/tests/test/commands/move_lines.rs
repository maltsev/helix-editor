use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_unicode_and_line_endings() -> anyhow::Result<()> {
    for ending in ["\n", "\r\n"] {
        for trailing_newline in [false, true] {
            let input = "short\n雪#[|🙂]#e\u{301}".replace('\n', ending);
            let output = "雪#[|🙂]#e\u{301}\nshort".replace('\n', ending);
            let suffix = if trailing_newline { ending } else { "" };
            test((
                format!("{input}{suffix}"),
                "<C-k>",
                format!("{output}{suffix}"),
                LineFeedHandling::AsIs,
            ))
            .await?;
            test((
                format!("{output}{suffix}"),
                "<C-j>",
                format!("{input}{suffix}"),
                LineFeedHandling::AsIs,
            ))
            .await?;
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_multiple_selections() -> anyhow::Result<()> {
    for (input, keys, output) in [
        // Adjacent selected lines move together; the primary selection stays primary.
        (
            "a\nb#(|b)#\n雪#[🙂|]#\nlong\nz",
            "<C-k>",
            "b#(|b)#\n雪#[🙂|]#\na\nlong\nz",
        ),
        (
            "a\nb#(|b)#\n雪#[🙂|]#\nlong\nz",
            "<C-j>",
            "a\nlong\nb#(|b)#\n雪#[🙂|]#\nz",
        ),
        // Multiple selections on the same line must not move it twice.
        ("a\nb#(|b)#c#[d|]#\nz", "<C-k>", "b#(|b)#c#[d|]#\na\nz"),
        ("a\nb#(|b)#c#[d|]#\nz", "<C-j>", "a\nz\nb#(|b)#c#[d|]#"),
        // Independent groups move even when another group is at the document edge.
        ("#[|a]#\nbb\n#(|雪)#\nz", "<C-k>", "#[|a]#\n#(|雪)#\nbb\nz"),
        (
            "a\n#[|b]#\n雪雪\n#(|z)#",
            "<C-j>",
            "a\n雪雪\n#[|b]#\n#(|z)#",
        ),
        // Disjoint selections can cover some of the same lines.
        (
            "a\n#[|bb\nc]#c#(d\nee|)#\nz",
            "<C-k>",
            "#[|bb\nc]#c#(d\nee|)#\na\nz",
        ),
        (
            "a\n#[|bb\nc]#c#(d\nee|)#\nz",
            "<C-j>",
            "a\nz\n#[|bb\nc]#c#(d\nee|)#",
        ),
        // Unequal line lengths in separate groups require independent offsets.
        (
            "a\n#[|bb]#\n雪雪雪\n#(|d)#\nz",
            "<C-k>",
            "#[|bb]#\na\n#(|d)#\n雪雪雪\nz",
        ),
        (
            "a\n#[|bb]#\n雪雪雪\n#(|d)#\nz",
            "<C-j>",
            "a\n雪雪雪\n#[|bb]#\nz\n#(|d)#",
        ),
    ] {
        test((input, keys, output, LineFeedHandling::AsIs)).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_document_edges() -> anyhow::Result<()> {
    for input in ["#[|]#", "#[|雪]#", "#[|雪\n🙂]#", "#[|雪\n]#", "a\n#[|]#"] {
        test((input, "<C-j>", input, LineFeedHandling::AsIs)).await?;
    }
    for input in ["#[|]#", "#[|雪]#", "#[|雪\n🙂]#", "#[|雪\n]#"] {
        test((input, "<C-k>", input, LineFeedHandling::AsIs)).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_linewise_and_blank_lines() -> anyhow::Result<()> {
    for (input, keys, output) in [
        ("a\n#[|bb\n]#cc\nz", "<C-k>", "#[|bb\n]#a\ncc\nz"),
        ("a\n#[|bb\n]#cc\nz", "<C-j>", "a\ncc\n#[|bb\n]#z"),
        // A selection of a final newline is clipped when moved to an unterminated line.
        ("a\n#[|bb\n]#z", "<C-j>", "a\nz\n#[|bb]#"),
        ("a\n#[|\n]#z\n", "<C-k>", "#[|\n]#a\nz\n"),
        ("a\n#[|\n]#z\n", "<C-j>", "a\nz\n#[|\n]#"),
    ] {
        test((input, keys, output, LineFeedHandling::AsIs)).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_select_mode_and_undo() -> anyhow::Result<()> {
    let input = "a\n雪#[|🙂]#\nz";
    test((input, "v<C-k><C-j>", input, LineFeedHandling::AsIs)).await?;
    // Commit the test fixture before checking the command's history entry.
    test((input, "<esc><C-k>u", input, LineFeedHandling::AsIs)).await?;
    test((
        input,
        "<esc><C-k>uU",
        "雪#[|🙂]#\na\nz",
        LineFeedHandling::AsIs,
    ))
    .await?;
    Ok(())
}
