use std::iter::FusedIterator;
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self { head: None }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        let mut len = 0;
        let mut me = &self.head;
        while let Some(node_rc) = me {
            me = &node_rc.as_ref().next;
            len += 1;
        }
        len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    #[must_use]
    pub fn push(&self, elem: T) -> Self {
        let next = self.head.clone();
        Self {
            head: Some(Rc::new(Node { elem, next })),
        }
    }

    #[must_use]
    fn split(&self) -> Option<(&T, &Link<T>)> {
        self.head.as_ref().map(|x| {
            let Node { elem, next } = x.as_ref();
            (elem, next)
        })
    }

    #[must_use]
    pub fn car(&self) -> Option<&T> {
        self.split().map(|(x, _)| x)
    }

    #[must_use]
    pub fn cdr(&self) -> Self {
        self.split()
            .map(|(_, y)| Self { head: y.clone() })
            .unwrap_or_default()
    }

    #[must_use]
    pub const fn iter(&self) -> Iter<'_, T> {
        Iter { link: &self.head }
    }
}

impl<T> Default for List<T> {
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, T> {
    link: &'a Link<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.link.as_ref().map(|r| {
            self.link = &r.next;
            &r.elem
        })
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let list = List::new().push(1).push(4).push(9).push(16);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 4);
        assert_eq!(list.car().unwrap(), &16);
        assert_eq!(list.cdr().car().unwrap(), &9);
        assert_eq!(list.cdr().cdr().car().unwrap(), &4);
        assert_eq!(list.cdr().cdr().cdr().car().unwrap(), &1);
        assert!(list.cdr().cdr().cdr().cdr().car().is_none());
    }

    #[test]
    fn iterating() {
        let list = List::new().push(1).push(4).push(9).push(16);
        let vec: Vec<_> = list.iter().copied().collect();
        assert_eq!(vec, [16, 9, 4, 1]);
    }
}
