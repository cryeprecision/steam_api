macro_rules! load_test_json {
    ($filename:literal) => {{
        let mut path = ::std::path::PathBuf::from(::std::env!("CARGO_MANIFEST_DIR"));
        path.push("test_resources");
        path.push($filename);
        let file = match ::std::fs::OpenOptions::new().read(true).open(path) {
            Err(err) => {
                println!("couldn't open test resource file `{}`", $filename);
                panic!("{:?}", err);
            }
            Ok(file) => file,
        };
        match ::serde_json::from_reader(file) {
            Err(err) => {
                println!(
                    "couldn't parse json from test resource file `{}`",
                    $filename
                );
                panic!("{:?}", err);
            }
            Ok(v) => v,
        }
    }};
}
