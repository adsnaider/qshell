use sh::sh;

fn main() {
    let mut out = String::new();
    sh!(echo hello world > {out});
    print!("{}", out);
    sh! {
        echo 1;
        echo 2;
        echo 3;
    }
}
