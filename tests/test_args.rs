extern crate serenity;
#[macro_use] extern crate matches;

use serenity::framework::standard::{Args, ArgError};

#[test]
fn single_with_empty_message() {
    let mut args = Args::new("", &["".to_string()]);
    assert_matches!(args.single::<String>().unwrap_err(), ArgError::Eos);

    let mut args = Args::new("", &[",".to_string()]);
    assert_matches!(args.single::<String>().unwrap_err(), ArgError::Eos);
}

#[test]
fn single_n_with_empty_message() {
    let args = Args::new("", &["".to_string()]);
    assert_matches!(args.single_n::<String>().unwrap_err(), ArgError::Eos);

    let args = Args::new("", &[",".to_string()]);
    assert_matches!(args.single_n::<String>().unwrap_err(), ArgError::Eos);
}

#[test]
fn single_quoted_with_empty_message() {
    let mut args = Args::new("", &["".to_string()]);
    assert_matches!(args.single_quoted::<String>().unwrap_err(), ArgError::Eos);

    let mut args = Args::new("", &[",".to_string()]);
    assert_matches!(args.single_quoted::<String>().unwrap_err(), ArgError::Eos);
}

#[test]
fn multiple_with_empty_message() {
    let args = Args::new("", &["".to_string()]);
    assert_matches!(args.multiple::<String>().unwrap_err(), ArgError::Eos);

    let args = Args::new("", &[",".to_string()]);
    assert_matches!(args.multiple::<String>().unwrap_err(), ArgError::Eos);
}

#[test]
fn multiple_quoted_with_empty_message() {
    let args = Args::new("", &["".to_string()]);
    assert_matches!(args.multiple_quoted::<String>().unwrap_err(), ArgError::Eos);

    let args = Args::new("", &[",".to_string()]);
    assert_matches!(args.multiple_quoted::<String>().unwrap_err(), ArgError::Eos);
}

#[test]
fn skip_with_empty_message() {
    let mut args = Args::new("", &["".to_string()]);
    assert_matches!(args.skip(), None);

    let mut args = Args::new("", &[",".to_string()]);
    assert_matches!(args.skip(), None);
}

#[test]
fn skip_for_with_empty_message() {
    let mut args = Args::new("", &["".to_string()]);
    assert_matches!(args.skip_for(0), None);

    let mut args = Args::new("", &["".to_string()]);
    assert_matches!(args.skip_for(5), None);

    let mut args = Args::new("", &[",".to_string()]);
    assert_matches!(args.skip_for(0), None);

    let mut args = Args::new("", &[",".to_string()]);
    assert_matches!(args.skip_for(5), None);
}

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

#[test]
fn single_quoted_chaining() {
    let mut args = Args::new(r#""1, 2" "2" """#, &[" ".to_string()]);

    assert_eq!(args.single_quoted::<String>().unwrap(), "1, 2");
    assert_eq!(args.single_quoted::<String>().unwrap(), "2");
    assert_eq!(args.single_quoted::<String>().unwrap(), "");
}

#[test]
fn single_quoted_and_single_chaining() {
    let mut args = Args::new(r#""1, 2" "2" "3" 4"#, &[" ".to_string()]);

    assert_eq!(args.single_quoted::<String>().unwrap(), "1, 2");
    assert!(args.single_n::<i32>().is_err());
    assert_eq!(args.single::<String>().unwrap(), "\"2\"");
    assert_eq!(args.single_quoted::<i32>().unwrap(), 3);
    assert_eq!(args.single::<i32>().unwrap(), 4);
}

#[test]
fn full_on_args() {
    let test_text = "Some text to ensure `full()` works.";
    let args = Args::new(test_text, &[" ".to_string()]);

    assert_eq!(args.full(), test_text);
}

#[test]
fn multiple_quoted_strings_one_delimiter() {
    let args = Args::new(r#""1, 2" "a" "3" 4 "5"#, &[" ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["1, 2", "a", "3", "4", "\"5"]);
}

#[test]
fn multiple_quoted_strings_with_multiple_delimiter() {
    let args = Args::new(r#""1, 2" "a","3"4 "5"#, &[" ".to_string(), ",".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["1, 2", "a", "3", "4", "\"5"]);
}

#[test]
fn multiple_quoted_strings_with_multiple_delimiters() {
    let args = Args::new(r#""1, 2" "a","3" """#, &[" ".to_string(), ",".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["1, 2", "a", "3", ""]);
}

#[test]
fn multiple_quoted_i32() {
    let args = Args::new(r#""1" "2" 3"#, &[" ".to_string()]);

    assert_eq!(args.multiple_quoted::<i32>().unwrap(), [1, 2, 3]);
}

#[test]
fn multiple_quoted_quote_appears_without_delimiter_in_front() {
    let args = Args::new(r#"hello, my name is cake" 2"#, &[",".to_string(), " ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello", "my", "name", "is", "cake\"", "2"]);
}

#[test]
fn multiple_quoted_single_quote() {
    let args = Args::new(r#"hello "2 b"#, &[",".to_string(), " ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello", "\"2 b"]);
}

#[test]
fn multiple_quoted_one_quote_pair() {
    let args = Args::new(r#"hello "2 b""#, &[",".to_string(), " ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello", "2 b"]);
}


#[test]
fn delimiter_before_multiple_quoted() {
    let args = Args::new(r#","hello, my name is cake" "2""#, &[",".to_string(), " ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello, my name is cake", "2"]);
}

#[test]
fn no_quote() {
    let args = Args::new("hello, my name is cake", &[",".to_string(), " ".to_string()]);

    assert_eq!(args.single_quoted_n::<String>().unwrap(), "hello");
}

#[test]
fn single_quoted_n() {
    let args = Args::new(r#""hello, my name is cake","test"#, &[",".to_string()]);

    assert_eq!(args.single_quoted_n::<String>().unwrap(), "hello, my name is cake");
    assert_eq!(args.single_quoted_n::<String>().unwrap(), "hello, my name is cake");
}

#[test]
fn multiple_quoted_starting_with_wrong_delimiter_in_first_quote() {
    let args = Args::new(r#""hello, my name is cake" "2""#, &[",".to_string(), " ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello, my name is cake", "2"]);
}

#[test]
fn multiple_quoted_with_one_correct_and_one_invalid_quote() {
    let args = Args::new(r#""hello, my name is cake" "2""#, &[",".to_string(), " ".to_string()]);

    assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello, my name is cake", "2"]);
}

#[test]
fn find_i32_one_one_byte_delimiter() {
    let mut args = Args::new("hello,my name is cake 2", &[" ".to_string()]);

    assert_eq!(args.find::<i32>().unwrap(), 2);
}

#[test]
fn find_i32_one_three_byte_delimiter() {
    let mut args = Args::new("hello,my name is cakeé2", &["é".to_string()]);

    assert_eq!(args.find::<i32>().unwrap(), 2);
}

#[test]
fn find_i32_multiple_delimiter_but_i32_not_last() {
    let mut args = Args::new("hello,my name is 2 cake", &[" ".to_string(), ",".to_string()]);

    assert_eq!(args.find::<i32>().unwrap(), 2);
}

#[test]
fn find_i32_multiple_delimiter() {
    let mut args = Args::new("hello,my name is cake 2", &[" ".to_string(), ",".to_string()]);

    assert_eq!(args.find::<i32>().unwrap(), 2);
}

#[test]
fn find_n_i32() {
    let mut args = Args::new("a 2", &[" ".to_string()]);

    assert_eq!(args.find_n::<i32>().unwrap(), 2);
    assert_eq!(args.find_n::<i32>().unwrap(), 2);
}

#[test]
fn skip() {
    let mut args = Args::new("1 2", &[" ".to_string()]);

    assert_eq!(args.skip().unwrap(), "1");
    assert_eq!(args.remaining(), 1);
    assert_eq!(args.single::<String>().unwrap(), "2");
}

#[test]
fn skip_for() {
    let mut args = Args::new("1 2 neko 100", &[" ".to_string()]);

    assert_eq!(args.skip_for(2).unwrap(), ["1", "2"]);
    assert_eq!(args.remaining(), 2);
    assert_eq!(args.single::<String>().unwrap(), "neko");
    assert_eq!(args.single::<String>().unwrap(), "100");
}

#[test]
fn len_with_one_delimiter() {
    let args = Args::new("1 2 neko 100", &[" ".to_string()]);

    assert_eq!(args.len(), 4);
    assert_eq!(args.remaining(), 4);
}

#[test]
fn len_multiple_quoted() {
    let args = Args::new(r#""hello, my name is cake" "2""#, &[" ".to_string()]);

    assert_eq!(args.len(), 2);
}

#[test]
fn remaining_len_before_and_after_single() {
    let mut args = Args::new("1 2", &[" ".to_string()]);

    assert_eq!(args.remaining(), 2);
    assert_eq!(args.single::<i32>().unwrap(), 1);
    assert_eq!(args.remaining(), 1);
    assert_eq!(args.single::<i32>().unwrap(), 2);
    assert_eq!(args.remaining(), 0);
}

#[test]
fn remaining_len_before_and_after_single_quoted() {
    let mut args = Args::new(r#""1" "2" "3""#, &[" ".to_string()]);

    assert_eq!(args.remaining(), 3);
    assert_eq!(args.single_quoted::<i32>().unwrap(), 1);
    assert_eq!(args.remaining(), 2);
    assert_eq!(args.single_quoted::<i32>().unwrap(), 2);
    assert_eq!(args.remaining(), 1);
    assert_eq!(args.single_quoted::<i32>().unwrap(), 3);
    assert_eq!(args.remaining(), 0);
}

#[test]
fn remaining_len_before_and_after_skip() {
    let mut args = Args::new("1 2", &[" ".to_string()]);

    assert_eq!(args.remaining(), 2);
    assert_eq!(args.skip().unwrap(), "1");
    assert_eq!(args.remaining(), 1);
    assert_eq!(args.skip().unwrap(), "2");
    assert_eq!(args.remaining(), 0);
}

#[test]
fn remaining_len_before_and_after_skip_empty_string() {
    let mut args = Args::new("", &[" ".to_string()]);

    assert_eq!(args.remaining(), 0);
    assert_eq!(args.skip(), None);
    assert_eq!(args.remaining(), 0);
}

#[test]
fn remaining_len_before_and_after_skip_for() {
    let mut args = Args::new("1 2", &[" ".to_string()]);

    assert_eq!(args.remaining(), 2);
    assert_eq!(args.skip_for(2), Some(vec!["1".to_string(), "2".to_string()]));
    assert_eq!(args.skip_for(2), None);
    assert_eq!(args.remaining(), 0);
}

#[test]
fn remaining_len_before_and_after_find() {
    let mut args = Args::new("a 2 6", &[" ".to_string()]);

    assert_eq!(args.remaining(), 3);
    assert_eq!(args.find::<i32>().unwrap(), 2);
    assert_eq!(args.remaining(), 2);
    assert_eq!(args.find::<i32>().unwrap(), 6);
    assert_eq!(args.remaining(), 1);
    assert_eq!(args.find::<String>().unwrap(), "a");
    assert_eq!(args.remaining(), 0);
    assert_matches!(args.find::<String>().unwrap_err(), ArgError::Eos);
    assert_eq!(args.remaining(), 0);
}

#[test]
fn remaining_len_before_and_after_find_n() {
    let mut args = Args::new("a 2 6", &[" ".to_string()]);

    assert_eq!(args.remaining(), 3);
    assert_eq!(args.find_n::<i32>().unwrap(), 2);
    assert_eq!(args.remaining(), 3);
}


#[test]
fn multiple_strings_with_one_delimiter() {
    let args = Args::new("hello, my name is cake 2", &[" ".to_string()]);

    assert_eq!(args.multiple::<String>().unwrap(), ["hello,", "my", "name", "is", "cake", "2"]);
}

#[test]
fn multiple_i32_with_one_delimiter() {
    let args = Args::new("1 2 3", &[" ".to_string()]);

    assert_eq!(args.multiple::<i32>().unwrap(), [1, 2, 3]);
}

#[test]
fn multiple_i32_with_one_delimiter_and_parse_error() {
    let args = Args::new("1 2 3 abc", &[" ".to_string()]);

    assert_matches!(args.multiple::<i32>().unwrap_err(), ArgError::Parse(_));
}

#[test]
fn multiple_i32_with_three_delimiters() {
    let args = Args::new("1 2 3", &[" ".to_string(), ",".to_string()]);

    assert_eq!(args.multiple::<i32>().unwrap(), [1, 2, 3]);
}

#[test]
fn single_after_failed_single() {
    let mut args = Args::new("b 2", &[" ".to_string()]);

    assert_matches!(args.single::<i32>().unwrap_err(), ArgError::Parse(_));
    // Test that `single` short-circuts on an error and leaves the source as is.
    assert_eq!(args.remaining(), 2);
    assert_eq!(args.single::<String>().unwrap(), "b");
    assert_eq!(args.single::<String>().unwrap(), "2");
}

#[test]
fn remaining_len_after_failed_single_quoted() {
    let mut args = Args::new("b a", &[" ".to_string()]);

    assert_eq!(args.remaining(), 2);
    // Same goes for `single_quoted` and the alike.
    assert_matches!(args.single_quoted::<i32>().unwrap_err(), ArgError::Parse(_));
    assert_eq!(args.remaining(), 2);
}
