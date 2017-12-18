extern crate serenity;

use serenity::framework::standard::Args;

#[test]
fn single_i32_with_2_bytes_long_delimiter() {
    let mut args = Args::new("1, 2", &[", ".to_string()]);

    assert_eq!(args.single::<i32>().unwrap(), 1);
    assert_eq!(args.single::<i32>().unwrap(), 2);
}

#[test]
fn single_i32_with_1_byte_long_delimiter_i32() {
    let mut args = Args::new("1,2", &[",".to_string()]);

    assert_eq!(args.single::<i32>().unwrap(), 1);
    assert_eq!(args.single::<i32>().unwrap(), 2);
}

#[test]
fn single_i32_with_wrong_char_after_first_arg() {
    let mut args = Args::new("1, 2", &[",".to_string()]);

    assert_eq!(args.single::<i32>().unwrap(), 1);
    assert!(args.single::<i32>().is_err());
}

#[test]
fn single_i32_with_one_character_being_3_bytes_long() {
    let mut args = Args::new("1★2", &["★".to_string()]);

    assert_eq!(args.single::<i32>().unwrap(), 1);
    assert_eq!(args.single::<i32>().unwrap(), 2);
}

#[test]
fn single_i32_with_untrimmed_whitespaces() {
    let mut args = Args::new(" 1, 2 ", &[",".to_string()]);

    assert!(args.single::<i32>().is_err());
}

#[test]
fn single_i32_n() {
    let args = Args::new("1,2", &[",".to_string()]);

    assert_eq!(args.single_n::<i32>().unwrap(), 1);
    assert_eq!(args.single_n::<i32>().unwrap(), 1);
}
