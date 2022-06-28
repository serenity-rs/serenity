#[cfg(feature = "collector")]
#[rustversion::since(1.61)]
pub fn retain_mut<T>(vec: &mut Vec<T>, f: impl FnMut(&mut T) -> bool) {
    vec.retain_mut(f);
}

#[cfg(feature = "collector")]
#[rustversion::not(since(1.61))]
pub fn retain_mut<T>(vec: &mut Vec<T>, mut f: impl FnMut(&mut T) -> bool) {
    let len = vec.len();
    let mut del = 0;
    {
        let v = &mut **vec;

        for i in 0..len {
            if !f(&mut v[i]) {
                del += 1;
            } else if del > 0 {
                v.swap(i - del, i);
            }
        }
    }

    if del > 0 {
        vec.truncate(len - del);
    }
}
