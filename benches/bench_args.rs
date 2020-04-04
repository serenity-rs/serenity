#![feature(test)]

#[cfg(test)]
mod benches {
    extern crate test;

    use serenity::framework::standard::{Args, Delimiter};
    use self::test::Bencher;

    #[bench]
    fn single_with_one_delimiter(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1,2", &[Delimiter::Single(',')]);
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_with_one_delimiter_and_long_string(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new(
                "1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25",
                &[Delimiter::Single(',')],
            );
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_with_three_delimiters(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new(
                "1,2 @3@4 5,",
                &[
                    Delimiter::Single(','),
                    Delimiter::Single(' '),
                    Delimiter::Single('@'),
                ],
            );
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_with_three_delimiters_and_long_string(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new(
                "1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,1,2 @3@4 5,",
                &[
                    Delimiter::Single(','),
                    Delimiter::Single(' '),
                    Delimiter::Single('@'),
                ],
            );
            args.single::<String>().unwrap();
        })
    }

    #[bench]
    fn single_quoted_with_one_delimiter(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new(r#""1","2""#, &[Delimiter::Single(',')]);
            args.single_quoted::<String>().unwrap();
        })
    }

    #[bench]
    fn iter_with_one_delimiter(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1,2,3,4,5,6,7,8,9,10", &[Delimiter::Single(',')]);
            args.iter::<String>().collect::<Result<Vec<_>, _>>().unwrap();
        })
    }

    #[bench]
    fn iter_with_three_delimiters(b: &mut Bencher) {
        b.iter(|| {
            let mut args = Args::new("1-2<3,4,5,6,7<8,9,10", &[Delimiter::Single(','), Delimiter::Single('-'), Delimiter::Single('<')]);
            args.iter::<String>().collect::<Result<Vec<_>, _>>().unwrap();
        })
    }
}
