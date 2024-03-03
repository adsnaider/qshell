use qshell::{cmd, sh};

#[test]
fn echo() {
    sh!(echo hello world);
    let mut out = String::new();
    sh!(echo hello world > {&mut out});
    assert_eq!(out, "hello world\n");
    out.clear()
}

#[test]
fn multiple() {
    cmd! {
        echo hello world;
        echo hello world;
    };
}

#[test]
fn literals() {
    let mut out = String::new();
    sh!(echo "hello world" > {&mut out});
    assert_eq!(out, "hello world\n");
    out.clear();

    sh!(echo 1 2 3u8 > {&mut out});
    assert_eq!(out, "1 2 3u8\n");
    out.clear();

    sh!(echo 1.23 > {&mut out});
    assert_eq!(out, "1.23\n");
    out.clear();

    sh!(echo true > {&mut out});
    assert_eq!(out, "true\n");
    out.clear()
}

#[test]
fn source_types() {
    let mut out = String::new();
    let hello = "hello world";
    sh!(cat < {hello} > {&mut out});
    assert_eq!(&out, "hello world");
    out.clear();

    let hello = "hello world".to_string();
    sh!(cat < {hello} > {&mut out});
    assert_eq!(&out, "hello world");
    out.clear();

    let hello = b"hello world".as_slice();
    sh!(cat < {hello} > {&mut out});
    assert_eq!(&out, "hello world");
    out.clear();

    let hello = b"hello world".to_vec();
    sh!(cat < {hello} > {&mut out});
    assert_eq!(&out, "hello world");
    out.clear();
}

#[test]
fn sink_types() {
    let mut out = String::new();
    sh!(echo "hello world" > {&mut out});
    assert_eq!(out, "hello world\n");
    out.clear();

    let mut out = Vec::new();
    sh!(echo "hello world" > {&mut out});
    assert_eq!(out, b"hello world\n");
    out.clear();
}
