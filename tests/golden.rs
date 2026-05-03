use std::fs;
use std::path::Path;

fn run_golden_test(test_file: &str) {
    let base = Path::new("tests/fixtures").join(test_file);
    let input = fs::read_to_string(&base).unwrap();

    let (out, err) = match hpre::process(&input) {
        Ok(out)  => (out, String::new()),
        Err(err) => (String::new(), err + "\n"),
    };

    for (which, got) in [("err", err), ("out", out)] {
        let test_file = base.with_added_extension(which);
        let sought = fs::read_to_string(&test_file).unwrap_or_default();
        pretty_assertions::assert_eq!(got, sought, "Mismatched {}", which);
    }
}

macro_rules! golden_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            run_golden_test(concat!(stringify!($name), ".hs"));
        }
    };
}

golden_test!(commas);
golden_test!(ditto0);
golden_test!(ditto1);
golden_test!(ditto2);
golden_test!(ditto3);
golden_test!(ditto4);
golden_test!(guards);
golden_test!(imports);
golden_test!(letditto);
golden_test!(qualified1);
golden_test!(qualified2);
golden_test!(syntax);
golden_test!(unicode);

