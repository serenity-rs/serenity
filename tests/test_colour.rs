extern crate serenity;

use serenity::utils::Colour;
use std::u32;

#[test]
fn new() {
    assert_eq!(Colour::new(1).0, 1);
    assert_eq!(Colour::new(u32::MIN).0, u32::MIN);
    assert_eq!(Colour::new(u32::MAX).0, u32::MAX);
}

#[test]
fn from_rgb() {
    assert_eq!(Colour::from_rgb(255, 0, 0).0, 0xFF0000);
    assert_eq!(Colour::from_rgb(0, 255, 0).0, 0x00FF00);
    assert_eq!(Colour::from_rgb(0, 0, 255).0, 0x0000FF);
}

#[test]
fn get_r() {
    assert_eq!(Colour::new(0x336123).get_r(), 0x33);
}

#[test]
fn get_g() {
    assert_eq!(Colour::new(0x336123).get_g(), 0x61);
}

#[test]
fn get_b() {
    assert_eq!(Colour::new(0x336123).get_b(), 0x23);
}

#[test]
fn get_tuple() {
    assert_eq!(Colour::new(0x336123).get_tuple(), (0x33, 0x61, 0x23));
}

#[test]
fn default() {
    assert_eq!(Colour::default().0, 0);
}

#[test]
fn from() {
    assert_eq!(Colour::from(7i32).0, 7);
    assert_eq!(Colour::from(7u32).0, 7);
    assert_eq!(Colour::from(7u64).0, 7);
}
