//! Talks to `qed mcp` (#105) the way an assistant does: JSON-RPC lines
//! over stdin and stdout. This also checks that nothing else is printed to
//! stdout, which would break the conversation.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

struct Session {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next_id: u64,
}

impl Session {
    fn start(site: &std::path::Path) -> Session {
        let mut child = Command::new(env!("CARGO_BIN_EXE_qed"))
            .arg("mcp")
            .arg(site)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut session = Session {
            child,
            input,
            output,
            next_id: 0,
        };
        let init = session.request(
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "test", "version": "1"}}),
        );
        assert_eq!(init["result"]["serverInfo"]["name"], "qed");
        // Short, so assistants don't cut it off; it points to the guide.
        let instructions = init["result"]["instructions"].as_str().unwrap();
        assert!(
            instructions.len() < 2000 && instructions.contains("guide tool"),
            "{instructions}"
        );
        session.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        let (guide, failed) = session.call("guide", json!({}));
        assert!(!failed && guide.contains(":::: columns") && guide.contains("redirect_from"));
        assert!(guide.contains("catppuccin-mocha") && guide.contains("book-open"));
        session
    }

    fn send(&mut self, message: Value) {
        writeln!(self.input, "{message}").unwrap();
        self.input.flush().unwrap();
    }

    /// Sends a request and returns its answer; every line on stdout must
    /// be a JSON-RPC message.
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let mut line = String::new();
            assert!(
                self.output.read_line(&mut line).unwrap() > 0,
                "server closed"
            );
            let message: Value = serde_json::from_str(&line)
                .unwrap_or_else(|_| panic!("not JSON-RPC on stdout: {line}"));
            if message["id"] == id {
                return message;
            }
        }
    }

    /// Calls a tool: its text, and whether it failed.
    fn call(&mut self, tool: &str, arguments: Value) -> (String, bool) {
        let reply = self.request("tools/call", json!({"name": tool, "arguments": arguments}));
        let result = &reply["result"];
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        (text, result["isError"] == true)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[test]
fn edits_a_page_with_the_apps_rules() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("site.toml"),
        "title = \"Test\"\nlanguage = \"de\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("pages")).unwrap();
    std::fs::write(
        root.join("pages/index.md"),
        "+++\ntitle = \"Home\"\n+++\n\nHi.\n",
    )
    .unwrap();
    qed_core::history::start(root).unwrap();
    let mut session = Session::start(root);

    let (info, _) = session.call("site_info", json!({}));
    assert!(info.contains("Language: de"), "{info}");

    let (page, failed) = session.call("read_page", json!({"file": "pages/index.md"}));
    assert!(!failed, "{page}");
    let stamp = page
        .lines()
        .next()
        .unwrap()
        .trim_start_matches("stamp: ")
        .to_string();
    let text = page.split_once("\n\n").unwrap().1.replace("Hi.", "Hello.");

    let (saved, failed) = session.call(
        "write_page",
        json!({"file": "pages/index.md", "text": text, "stamp": stamp}),
    );
    assert!(!failed, "{saved}");
    assert!(
        std::fs::read_to_string(root.join("pages/index.md"))
            .unwrap()
            .contains("Hello.")
    );

    // The same stamp again: the file changed since that read.
    let (refused, failed) = session.call(
        "write_page",
        json!({"file": "pages/index.md", "text": "+++\ntitle = \"X\"\n+++\n", "stamp": stamp}),
    );
    assert!(
        failed && refused.contains("changed since it was read"),
        "{refused}"
    );

    // Nothing outside the site folder.
    let (refused, failed) = session.call("read_page", json!({"file": "../outside.md"}));
    assert!(failed, "{refused}");

    // The change is in the history, and a note closes it.
    let (saved, _) = session.call("save_version", json!({"note": "Greeting"}));
    assert!(saved.contains("Greeting"), "{saved}");
    let versions = qed_core::history::versions(root, None).unwrap();
    assert_eq!(versions[0].summary, "Greeting");

    let (built, failed) = session.call("build_site", json!({}));
    assert!(!failed && built.starts_with("Built 1 pages"), "{built}");
}

#[test]
fn reaches_static_files_the_footer_and_the_settings() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("site");
    std::fs::create_dir_all(root.join("pages")).unwrap();
    std::fs::write(root.join("site.toml"), "title = \"Test\"\n").unwrap();
    std::fs::write(
        root.join("pages/index.md"),
        "+++\ntitle = \"Home\"\n+++\n\n:icon-envelope: Mail\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("pages/blog")).unwrap();
    std::fs::write(root.join("pages/blog.md"), "+++\ntitle = \"Blog\"\n+++\n").unwrap();
    let old = dir.path().join("old");
    std::fs::create_dir_all(old.join("talks")).unwrap();
    std::fs::write(old.join("talks/sub8.html"), "<html>slides</html>").unwrap();
    std::fs::write(old.join("33C3.pdf"), "%PDF").unwrap();
    let mut session = Session::start(&root);

    // A post that keeps its old name.
    let (made, failed) = session.call(
        "create_page",
        json!({"parent": "blog", "title": "Panoptikum Blog", "date": "2016-10-02",
               "slug": "2016-10-02-beginn"}),
    );
    assert!(
        !failed && made.starts_with("Made /blog/2016-10-02-beginn"),
        "{made}"
    );
    let (list, _) = session.call("list_pages", json!({}));
    assert!(list.starts_with("3 pages, 1 of them posts."), "{list}");

    // Files at their old addresses, a folder with everything in it.
    let (imported, failed) = session.call(
        "import_file",
        json!({"path": old.join("33C3.pdf"), "library": "static", "folder": "pdf"}),
    );
    assert!(!failed, "{imported}");
    let (imported, failed) = session.call(
        "import_file",
        json!({"path": old.join("talks"), "library": "static"}),
    );
    assert!(!failed, "{imported}");
    assert!(root.join("static/pdf/33C3.pdf").is_file());
    assert!(root.join("static/talks/sub8.html").is_file());
    let (refused, failed) = session.call(
        "import_file",
        json!({"path": old.join("33C3.pdf"), "library": "assets", "folder": "pdf"}),
    );
    assert!(failed, "{refused}");

    // The footer doesn't exist yet: it reads empty, and writing makes it.
    let (footer, failed) = session.call("read_page", json!({"file": "footer.md"}));
    assert!(!failed, "{footer}");
    let stamp = footer.lines().next().unwrap().trim_start_matches("stamp: ");
    let (saved, failed) = session.call(
        "write_page",
        json!({"file": "footer.md", "text": "+++\ntitle = \"Footer\"\n+++\n\n© 2026\n",
               "stamp": stamp}),
    );
    assert!(!failed, "{saved}");
    let (list, _) = session.call("list_pages", json!({}));
    assert!(list.contains("footer.md · the footer"), "{list}");

    // Settings: a broken text is refused, a good one saved.
    let (settings, _) = session.call("read_settings", json!({}));
    let stamp = settings
        .lines()
        .next()
        .unwrap()
        .trim_start_matches("stamp: ");
    let (refused, failed) = session.call(
        "write_settings",
        json!({"text": "title = ", "stamp": stamp}),
    );
    assert!(failed, "{refused}");
    let (saved, failed) = session.call(
        "write_settings",
        json!({"text": "title = \"Test\"\nprofiles = [\"https://fosstodon.org/@x\"]\n",
               "stamp": stamp}),
    );
    assert!(!failed, "{saved}");

    let (built, failed) = session.call("build_site", json!({}));
    assert!(!failed, "{built}");
    assert!(built.contains("uses the icon \"envelope\""), "{built}");
    let home = std::fs::read_to_string(root.join("_site/index.html")).unwrap();
    assert!(
        home.contains(r#"rel="me" href="https://fosstodon.org/@x""#),
        "{home}"
    );
    assert!(root.join("_site/pdf/33C3.pdf").is_file());
}
