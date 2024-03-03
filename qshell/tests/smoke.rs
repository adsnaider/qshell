use qshell::{cmd, sh};

#[test]
fn echo() {
    sh!(echo hello world);
    let mut out = String::new();
    sh!(echo hello world > {out});
    assert_eq!(out, "hello world\n");
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
    sh!(echo "hello world" > {out});
    assert_eq!(out, "hello world\n");

    sh!(echo 1 2 3u8 > {out});
    assert_eq!(out, "1 2 3u8\n");

    sh!(echo 1.23 > {out});
    assert_eq!(out, "1.23\n");

    sh!(echo true > {out});
    assert_eq!(out, "true\n");
}
