use std::process::Command;

/// Check that buffers are flushed properly.
///
/// At some point during the v2 alpha/beta period `stdout` was wrapped with a
/// [buffered writer](https://doc.rust-lang.org/std/io/struct.BufWriter.html).
/// It was never explicitly flushed, but its `Drop` impl does that so it should
/// have worked. However, the buffer was created in `main` and kept in scope
/// until the end of the function. Since `std::process::exit` was called
/// directly in `main`, the buffer's `Drop` impl was not called before the
/// process exited. Version 2.0.0 was released with this issue.
///
/// See: https://github.com/allenap/rust-petname/issues/109
#[test]
fn test_buffers_are_flushed() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--words=2")
        .arg("--count=3") // Not enough to make `BufWriter` flush its buffer.
        .output()
        .expect("Failed to run petname CLI");
    assert!(output.status.success(), "Process did not exit successfully");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Expected 3 petnames, got {}", lines.len());
    for line in lines {
        let words: Vec<&str> = line.split('-').filter(|word| !word.is_empty()).collect();
        assert_eq!(words.len(), 2, "Expected 2 words in petname, got {}", words.len());
    }
}
