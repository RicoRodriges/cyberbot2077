pub fn new_vec<T: Copy>(a: &[T], b: &[T]) -> Vec<T> {
    let mut r = Vec::with_capacity(a.len() + b.len());
    r.extend(a);
    r.extend(b);
    return r;
}

pub fn is_part_of<T: Eq>(main: &[T], part: &[T]) -> bool {
    if main.is_empty() || part.is_empty() || part.len() > main.len() {
        return false;
    }

    let start = &part[0];
    for (i, v) in main.iter().enumerate() {
        if part.len() > main.len() - i {
            return false;
        }
        if v == start && part == &main[i..(i + part.len())] {
            return true;
        }
    }
    return false;
}

pub fn union_point<T: Eq>(c1: &[T], c2: &[T]) -> Option<usize> {
    if c1.is_empty() || c2.is_empty() {
        return None;
    }

    let start = &c2[0];
    for (i, v) in c1.iter().enumerate() {
        let c2_len = usize::min(c1.len() - i, c2.len());
        if v == start && &c1[i..] == &c2[..c2_len] {
            return Some(i);
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use crate::util::{is_part_of, new_vec, union_point};

    #[test]
    fn test_new_vec() {
        let v1 = new_vec::<u8>(&[], &[]);
        assert_eq!(true, v1.is_empty());
        assert_eq!(0, v1.capacity());

        let v2 = new_vec::<u8>(&[1, 2], &[]);
        assert_eq!(vec![1, 2], v2);
        assert_eq!(2, v2.capacity());

        let v3 = new_vec::<u8>(&[], &[1, 2]);
        assert_eq!(vec![1, 2], v3);
        assert_eq!(2, v3.capacity());

        let v4 = new_vec::<u8>(&[1, 2], &[3, 4]);
        assert_eq!(vec![1, 2, 3, 4], v4);
        assert_eq!(4, v4.capacity());
    }

    #[test]
    fn test_is_part_of() {
        assert_eq!(true, is_part_of(&vec![0, 1, 2, 3, 4], &vec![0, 1, 2, 3, 4]));
        assert_eq!(true, is_part_of(&vec![0, 1, 2, 3, 4], &vec![0, 1, 2, 3]));
        assert_eq!(true, is_part_of(&vec![0, 1, 2, 3, 4], &vec![1, 2, 3, 4]));
        assert_eq!(true, is_part_of(&vec![0, 1, 2, 3, 4], &vec![1, 2, 3]));
        assert_eq!(true, is_part_of(&vec![0, 1, 2, 3, 4], &vec![2]));

        assert_eq!(false, is_part_of(&vec![0, 1, 2], &vec![2, 3]));
        assert_eq!(false, is_part_of(&vec![0, 1, 2], &vec![3]));
        assert_eq!(false, is_part_of(&vec![0, 1, 2], &vec![0, 1, 2, 3]));
    }

    #[test]
    fn test_union_point() {
        assert_eq!(Some(2), union_point(&vec![0, 1, 2], &vec![2, 3, 4]));
        assert_eq!(Some(1), union_point(&vec![0, 1, 2], &vec![1, 2, 3]));
        assert_eq!(Some(1), union_point(&vec![0, 1, 2], &vec![1, 2]));

        assert_eq!(None, union_point(&vec![0, 1, 2], &vec![1, 3, 2]));
        assert_eq!(None, union_point(&vec![0, 1, 2], &vec![3, 4, 5]));
    }
}