use config_cli::{Capacity, label, load, save_new};
#[test]
fn historical_bytes_keep_their_meaning() {
    assert_eq!(load(&[1, 0]), Ok(Capacity::Unlimited));
    for value in 1..=u8::MAX {
        assert_eq!(load(&[1, value]), Ok(Capacity::Bounded(value)));
        assert_eq!(load(&[2, value]), Ok(Capacity::Bounded(value)));
        assert_eq!(save_new(value), Ok([2, value]));
    }
}
#[test]
fn rejects_corrupt_or_unsupported_records() {
    for bytes in [&[][..], &[1], &[1, 1, 0], &[2, 0], &[3, 1], &[0, 1]] {
        assert!(load(bytes).is_err(), "{bytes:?}");
    }
    assert!(save_new(0).is_err());
}
#[test]
fn simple_label_stays_usable() { assert_eq!(label("a", "b"), "a: b"); }
#[test]
fn stdout_reports_legacy_unlimited() {
    let path = std::env::temp_dir().join(format!("config-cli-oracle-{}", std::process::id()));
    std::fs::write(&path, [1, 0]).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_config_cli")).arg(&path).output().unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "capacity: Unlimited\n");
}
