use std::rc::Rc;

#[derive(Debug)]
pub struct LinkedList<T> {
    node: Option<Rc<LinkedListNode<T>>>,
}

impl<T> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> LinkedList<T> {
    fn new_after(item: T, prev: Option<Rc<LinkedListNode<T>>>) -> Self {
        Self {
            node: Some(Rc::new(LinkedListNode { item, prev })),
        }
    }

    pub fn empty() -> Self {
        Self { node: None }
    }

    pub fn append(&self, item: T) -> Self {
        Self::new_after(item, self.node.clone())
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut node = &self.node;
        let mut buf = Vec::new();

        loop {
            let Some(n) = node else {
                break;
            };

            buf.push(n.item.clone());

            node = &n.prev;
        }

        buf
    }
}

#[derive(Debug)]
struct LinkedListNode<T> {
    item: T,
    prev: Option<Rc<Self>>,
}
