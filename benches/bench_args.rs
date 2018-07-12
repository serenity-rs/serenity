#![feature(test)]

#[cfg(test)]
mod benches {
    extern crate serenity;
    extern crate test;

    use self::serenity::framework::standard::Args;
    use self::test::Bencher;

    #[bench]
    fn single_with_one_delimiter(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1,2", &[",".to_string()]);
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_with_one_delimiter_and_long_string(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25", &[",".to_string()]);
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_with_three_delimiters(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1,2 @3@4 5,", &[",".to_string(), " ".to_string(), "@".to_string()]);
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_with_three_delimiters_and_long_string(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,", &[",".to_string(), " ".to_string(), "@".to_string()]);
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_quoted_with_one_delimiter(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new(r#""1","2""#, &[",".to_string()]);
            args.single_quoted::<String>().unwrap();
        })
    }

    #[bench]
    fn multiple_with_one_delimiter(b: &mut Bencher) {
        b.iter(|| {
            let args = Args::new("1,2,3,4,5,6,7,8,9,10", &[",".to_string()]);
            args.multiple::<String>().unwrap();
        })
    }

    #[bench]
    fn multiple_with_three_delimiters(b: &mut Bencher) {
        b.iter(|| {
            let args = Args::new("1-2<3,4,5,6,7<8,9,10", &[",".to_string(), "-".to_string(), "<".to_string()]);
            args.multiple::<String>().unwrap();
        })
    }
}