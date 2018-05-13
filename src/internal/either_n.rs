use futures::{
    Future,
    Poll,
};

#[allow(dead_code)]
pub enum Either3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

impl<A, B, C> Future for Either3<A, B, C>
    where A: Future,
        B: Future<Item = A::Item, Error = A::Error>,
        C: Future<Item = A::Item, Error = A::Error> {
    type Item = A::Item;
    type Error = A::Error;

    fn poll(&mut self) -> Poll<A::Item, A::Error> {
        match *self {
            Either3::A(ref mut a) => a.poll(),
            Either3::B(ref mut b) => b.poll(),
            Either3::C(ref mut c) => c.poll(),
        }
    }
}

#[allow(dead_code)]
pub enum Either4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
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
            Either4::A(ref mut a) => a.poll(),
            Either4::B(ref mut b) => b.poll(),
            Either4::C(ref mut c) => c.poll(),
            Either4::D(ref mut d) => d.poll(),
        }
    }
}