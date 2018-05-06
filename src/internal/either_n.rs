use futures::{
    Future,
    Poll,
};

pub enum Either3<A, B, C> {
    One(A),
    Two(B),
    Three(C),
}

impl<A, B, C> Future for Either3<A, B, C>
    where A: Future,
        B: Future<Item = A::Item, Error = A::Error>,
        C: Future<Item = A::Item, Error = A::Error> {
    type Item = A::Item;
    type Error = A::Error;

    fn poll(&mut self) -> Poll<A::Item, A::Error> {
        match *self {
            Either3::One(ref mut a) => a.poll(),
            Either3::Two(ref mut b) => b.poll(),
            Either3::Three(ref mut c) => c.poll(),
        }
    }
}

pub enum Either4<A, B, C, D> {
    One(A),
    Two(B),
    Three(C),
    Four(D),
}

impl<A, B, C, D> Future for Either4<A, B, C, D>
    where A: Future,
        B: Future<Item = A::Item, Error = A::Error>,
        C: Future<Item = A::Item, Error = A::Error>,
        D: Future<Item = A::Item, Error = A::Error> {
    type Item = A::Item;
    type Error = A::Error;

    fn poll(&mut self) -> Poll<A::Item, A::Error> {
        match *self {
            Either4::One(ref mut a) => a.poll(),
            Either4::Two(ref mut b) => b.poll(),
            Either4::Three(ref mut c) => c.poll(),
            Either4::Four(ref mut d) => d.poll(),
        }
    }
}