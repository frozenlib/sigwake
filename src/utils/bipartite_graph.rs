#![allow(unused)]

use std::fmt::{self, Debug};

use derive_ex::Ex;
use slabmap::SlabMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct XKey(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YKey(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Edge<E> {
    x: XKey,
    y: YKey,
    x_prev: Option<usize>,
    x_next: Option<usize>,
    y_prev: Option<usize>,
    y_next: Option<usize>,
    data: E,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node<T> {
    data: T,
    head: Option<usize>,
}
impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self { data, head: None }
    }
}

#[derive(Debug, Clone)]
pub struct BipartiteGraph<X = (), Y = (), E = ()> {
    xs: SlabMap<Node<X>>,
    ys: SlabMap<Node<Y>>,
    es: SlabMap<Edge<E>>,
}
impl<X, Y, E> BipartiteGraph<X, Y, E> {
    pub fn new() -> Self {
        Self {
            xs: SlabMap::new(),
            ys: SlabMap::new(),
            es: SlabMap::new(),
        }
    }
    pub fn insert_x(&mut self, data: X) -> XKey {
        XKey(self.xs.insert(Node::new(data)))
    }
    pub fn insert_y(&mut self, data: Y) -> YKey {
        YKey(self.ys.insert(Node::new(data)))
    }
    pub fn insert_edge(&mut self, x: XKey, y: YKey, data: E) {
        let entry = Edge {
            x,
            y,
            x_prev: None,
            x_next: self.xs[x.0].head,
            y_prev: None,
            y_next: self.ys[y.0].head,
            data,
        };
        let e = self.es.insert(entry);
        if let Some(x_next) = self.xs[x.0].head {
            self.es[x_next].x_prev = Some(e);
        }
        if let Some(y_next) = self.ys[y.0].head {
            self.es[y_next].y_prev = Some(e);
        }
        self.xs[x.0].head = Some(e);
        self.ys[y.0].head = Some(e);
    }
    fn remove_edge(&mut self, e: usize) {
        let Edge {
            x,
            y,
            x_prev,
            x_next,
            y_prev,
            y_next,
            ..
        } = self.es[e];

        if let Some(x_prev) = x_prev {
            self.es[x_prev].x_next = x_next;
        } else {
            self.xs[x.0].head = x_next;
        }
        if let Some(x_next) = x_next {
            self.es[x_next].x_prev = x_prev;
        }

        if let Some(y_prev) = y_prev {
            self.es[y_prev].y_next = y_next;
        } else {
            self.ys[y.0].head = y_next;
        }
        if let Some(y_next) = y_next {
            self.es[y_next].y_prev = y_prev;
        }
        self.es.remove(e);
    }

    pub fn clear(&mut self) {
        self.xs.clear();
        self.ys.clear();
        self.es.clear();
    }

    pub fn remove_x(&mut self, id: XKey) {
        while let Some(e) = self.xs[id.0].head {
            self.remove_edge(e);
        }
        self.xs.remove(id.0);
    }

    pub fn remove_y(&mut self, id: YKey) {
        while let Some(e) = self.ys[id.0].head {
            self.remove_edge(e);
        }
        self.ys.remove(id.0);
    }

    pub fn get_x(&self, x: XKey) -> Option<&X> {
        self.xs.get(x.0).map(|node| &node.data)
    }
    pub fn get_x_mut(&mut self, x: XKey) -> Option<&mut X> {
        self.xs.get_mut(x.0).map(|node| &mut node.data)
    }
    pub fn get_y(&self, y: YKey) -> Option<&Y> {
        self.ys.get(y.0).map(|node| &node.data)
    }
    pub fn get_y_mut(&mut self, y: YKey) -> Option<&mut Y> {
        self.ys.get_mut(y.0).map(|node| &mut node.data)
    }

    pub fn contains_x(&self, x: XKey) -> bool {
        self.xs.contains_key(x.0)
    }
    pub fn contains_y(&self, y: YKey) -> bool {
        self.ys.contains_key(y.0)
    }

    pub fn xs_from_y(&self, y: YKey) -> impl Iterator<Item = (XKey, &E)> + '_ {
        XsFromY {
            g: self,
            e: self.ys[y.0].head,
        }
    }
    pub fn ys_from_x(&self, x: XKey) -> impl Iterator<Item = (YKey, &E)> + '_ {
        YsFromX {
            g: self,
            e: self.xs[x.0].head,
        }
    }

    pub fn xs(&self) -> impl Iterator<Item = (XKey, &X)> + '_ {
        self.xs.iter().map(|(k, v)| (XKey(k), &v.data))
    }
    pub fn ys(&self) -> impl Iterator<Item = (YKey, &Y)> + '_ {
        self.ys.iter().map(|(k, v)| (YKey(k), &v.data))
    }
    pub fn edges(&self) -> impl Iterator<Item = (XKey, YKey, &E)> + '_ {
        self.es.values().map(|edge| (edge.x, edge.y, &edge.data))
    }
}
struct XsFromY<'a, X, Y, E> {
    g: &'a BipartiteGraph<X, Y, E>,
    e: Option<usize>,
}
impl<'a, X, Y, E> Iterator for XsFromY<'a, X, Y, E> {
    type Item = (XKey, &'a E);
    fn next(&mut self) -> Option<Self::Item> {
        let e = &self.g.es[self.e?];
        let x = e.x;
        self.e = e.y_next;
        Some((x, &e.data))
    }
}

struct YsFromX<'a, X, Y, E> {
    g: &'a BipartiteGraph<X, Y, E>,
    e: Option<usize>,
}
impl<'a, X, Y, E> Iterator for YsFromX<'a, X, Y, E> {
    type Item = (YKey, &'a E);
    fn next(&mut self) -> Option<Self::Item> {
        let e = &self.g.es[self.e?];
        let y = e.y;
        self.e = e.x_next;
        Some((y, &e.data))
    }
}

#[cfg(test)]
mod tests {
    use proptest::sample::Index;
    use test_strategy::{Arbitrary, proptest};

    use super::*;

    trait BipartiteGraphApi<X, Y, E> {
        fn insert_x(&mut self, data: X) -> XKey;
        fn insert_y(&mut self, data: Y) -> YKey;
        fn insert_edge(&mut self, x: XKey, y: YKey, data: E);
        fn clear(&mut self);
        fn remove_x(&mut self, id: XKey);
        fn remove_y(&mut self, id: YKey);
        fn xs_from_y<'a>(&'a self, y: YKey) -> impl Iterator<Item = (XKey, &'a E)> + 'a
        where
            E: 'a;
        fn ys_from_x<'a>(&'a self, x: XKey) -> impl Iterator<Item = (YKey, &'a E)> + 'a
        where
            E: 'a;
        fn xs<'a>(&'a self) -> impl Iterator<Item = (XKey, &'a X)> + 'a
        where
            X: 'a;
        fn ys<'a>(&'a self) -> impl Iterator<Item = (YKey, &'a Y)> + 'a
        where
            Y: 'a;
        fn edges<'a>(&'a self) -> impl Iterator<Item = (XKey, YKey, &'a E)> + 'a
        where
            E: 'a;
    }

    impl<X, Y, E> BipartiteGraphApi<X, Y, E> for BipartiteGraph<X, Y, E> {
        fn insert_x(&mut self, data: X) -> XKey {
            self.insert_x(data)
        }

        fn insert_y(&mut self, data: Y) -> YKey {
            self.insert_y(data)
        }

        fn insert_edge(&mut self, x: XKey, y: YKey, data: E) {
            self.insert_edge(x, y, data);
        }

        fn clear(&mut self) {
            self.clear();
        }

        fn remove_x(&mut self, id: XKey) {
            self.remove_x(id);
        }

        fn remove_y(&mut self, id: YKey) {
            self.remove_y(id);
        }

        fn xs_from_y<'a>(&'a self, y: YKey) -> impl Iterator<Item = (XKey, &'a E)> + 'a
        where
            E: 'a,
        {
            self.xs_from_y(y)
        }

        fn ys_from_x<'a>(&'a self, x: XKey) -> impl Iterator<Item = (YKey, &'a E)> + 'a
        where
            E: 'a,
        {
            self.ys_from_x(x)
        }

        fn xs<'a>(&'a self) -> impl Iterator<Item = (XKey, &'a X)> + 'a
        where
            X: 'a,
        {
            self.xs()
        }

        fn ys<'a>(&'a self) -> impl Iterator<Item = (YKey, &'a Y)> + 'a
        where
            Y: 'a,
        {
            self.ys()
        }

        fn edges<'a>(&'a self) -> impl Iterator<Item = (XKey, YKey, &'a E)> + 'a
        where
            E: 'a,
        {
            self.edges()
        }
    }

    struct EdgeR<E> {
        x: usize,
        y: usize,
        data: E,
    }

    struct BipartiteGraphReferenceImpl<X, Y, E> {
        xs: SlabMap<X>,
        ys: SlabMap<Y>,
        es: Vec<EdgeR<E>>,
    }
    impl<X, Y, E> BipartiteGraphReferenceImpl<X, Y, E> {
        fn new() -> Self {
            Self {
                xs: SlabMap::new(),
                ys: SlabMap::new(),
                es: Vec::new(),
            }
        }
    }
    impl<X, Y, E> BipartiteGraphApi<X, Y, E> for BipartiteGraphReferenceImpl<X, Y, E> {
        fn insert_x(&mut self, data: X) -> XKey {
            XKey(self.xs.insert(data))
        }

        fn insert_y(&mut self, data: Y) -> YKey {
            YKey(self.ys.insert(data))
        }

        fn insert_edge(&mut self, x: XKey, y: YKey, data: E) {
            self.es.push(EdgeR {
                x: x.0,
                y: y.0,
                data,
            });
        }

        fn clear(&mut self) {
            self.xs.clear();
            self.ys.clear();
            self.es.clear();
        }

        fn remove_x(&mut self, id: XKey) {
            self.es.retain(|e| e.x != id.0);
            self.xs.remove(id.0);
        }

        fn remove_y(&mut self, id: YKey) {
            self.es.retain(|e| e.y != id.0);
            self.ys.remove(id.0);
        }

        fn xs_from_y<'a>(&'a self, y: YKey) -> impl Iterator<Item = (XKey, &'a E)> + 'a
        where
            E: 'a,
        {
            self.es.iter().filter_map(move |e| {
                if e.y == y.0 {
                    Some((XKey(e.x), &e.data))
                } else {
                    None
                }
            })
        }

        fn ys_from_x<'a>(&'a self, x: XKey) -> impl Iterator<Item = (YKey, &'a E)> + 'a
        where
            E: 'a,
        {
            self.es.iter().filter_map(move |e| {
                if e.x == x.0 {
                    Some((YKey(e.y), &e.data))
                } else {
                    None
                }
            })
        }

        fn xs<'a>(&'a self) -> impl Iterator<Item = (XKey, &'a X)> + 'a
        where
            X: 'a,
        {
            self.xs.iter().map(|(k, v)| (XKey(k), v))
        }

        fn ys<'a>(&'a self) -> impl Iterator<Item = (YKey, &'a Y)> + 'a
        where
            Y: 'a,
        {
            self.ys.iter().map(|(k, v)| (YKey(k), v))
        }

        fn edges<'a>(&'a self) -> impl Iterator<Item = (XKey, YKey, &'a E)> + 'a
        where
            E: 'a,
        {
            self.es.iter().map(|e| (XKey(e.x), YKey(e.y), &e.data))
        }
    }

    #[derive(Debug, Copy, Clone, Arbitrary)]
    enum Action {
        InsertX,
        InsertY,
        InsertEdge(Index, Index),
        RemoveX(Index),
        RemoveY(Index),
        Clear,
    }
    impl Action {
        fn resolve(self, g: &BipartiteGraph<usize, usize, usize>) -> Option<ActionResolved> {
            Some(match self {
                Action::InsertX => ActionResolved::InsertX,
                Action::InsertY => ActionResolved::InsertY,
                Action::InsertEdge(x, y) => ActionResolved::InsertEdge(
                    XKey(resolve_index(x, &g.xs)?),
                    YKey(resolve_index(y, &g.ys)?),
                ),
                Action::RemoveX(x) => ActionResolved::RemoveX(XKey(resolve_index(x, &g.xs)?)),
                Action::RemoveY(y) => ActionResolved::RemoveY(YKey(resolve_index(y, &g.ys)?)),
                Action::Clear => ActionResolved::Clear,
            })
        }
    }

    #[derive(Debug, Copy, Clone)]
    enum ActionResolved {
        InsertX,
        InsertY,
        InsertEdge(XKey, YKey),
        RemoveX(XKey),
        RemoveY(YKey),
        Clear,
    }
    impl ActionResolved {
        fn apply(
            self,
            g: &mut impl BipartiteGraphApi<usize, usize, usize>,
            data: usize,
            log: bool,
        ) -> Option<()> {
            match self {
                Self::InsertX => {
                    let key = g.insert_x(data);
                    if log {
                        println!("insert_x({data})->{key:?}");
                    }
                }
                Self::InsertY => {
                    let key = g.insert_y(data);
                    if log {
                        println!("insert_y({data})->{key:?}");
                    }
                }
                Self::InsertEdge(x, y) => {
                    g.insert_edge(x, y, data);
                    if log {
                        println!("insert_edge({x:?},{y:?},{data})");
                    }
                }
                Self::RemoveX(x) => {
                    g.remove_x(x);
                    if log {
                        println!("remove_x({x:?})");
                    }
                }
                Self::RemoveY(y) => {
                    g.remove_y(y);
                    if log {
                        println!("remove_y({y:?})");
                    }
                }
                Self::Clear => {
                    g.clear();
                    if log {
                        println!("clear()");
                    }
                }
            }
            Some(())
        }
    }
    fn resolve_index<T>(index: Index, m: &SlabMap<T>) -> Option<usize> {
        if m.is_empty() {
            None
        } else {
            m.keys().nth(index.index(m.len()))
        }
    }

    #[proptest]
    fn prop_test(actions: Vec<Action>) {
        let mut g = BipartiteGraph::new();
        let mut g_ref = BipartiteGraphReferenceImpl::new();

        println!("=============================");
        for (data, action) in actions.into_iter().enumerate() {
            let Some(action) = action.resolve(&g) else {
                continue;
            };
            action.apply(&mut g, data, true);
            action.apply(&mut g_ref, data, false);
            let xs = to_vec_sorted(g.xs());
            let ys = to_vec_sorted(g.ys());
            let es = to_vec_sorted(g.edges());
            let xs_ref = to_vec_sorted(g_ref.xs());
            let ys_ref = to_vec_sorted(g_ref.ys());
            let es_ref = to_vec_sorted(g_ref.edges());
            if (xs != xs_ref || ys != ys_ref || es != es_ref) {
                println!();
                println!("expected:");
                println!("xs={xs_ref:?}");
                println!("ys={ys_ref:?}");
                println!("es={es_ref:?}");
                println!("=============================");
            }
            assert_eq!(xs, xs_ref, "xs");
            assert_eq!(ys, ys_ref, "ys");
            assert_eq!(es, es_ref, "es");

            for (x, _) in g.xs() {
                let es = to_vec_sorted(g.ys_from_x(x));
                let es_ref = to_vec_sorted(g_ref.ys_from_x(x));
                assert_eq!(es, es_ref, "ys_from(x = {x:?})");
            }
            for (y, _) in g.ys() {
                let es = to_vec_sorted(g.xs_from_y(y));
                let es_ref = to_vec_sorted(g_ref.xs_from_y(y));
                assert_eq!(es, es_ref, "xs_from(y = {y:?})");
            }
        }
    }
    fn to_vec_sorted<T: Ord>(xs: impl Iterator<Item = T>) -> Vec<T> {
        let mut xs = xs.collect::<Vec<_>>();
        xs.sort();
        xs
    }

    #[test]
    fn insert_edge_1() {
        let mut g = BipartiteGraph::new();
        let x = g.insert_x(10);
        let y = g.insert_y(20);
        g.insert_edge(x, y, 30);
        assert_eq!(to_vec_sorted(g.edges()), vec![(x, y, &30)], "edges");
        assert_eq!(to_vec_sorted(g.xs_from_y(y)), vec![(x, &30)], "xs_from_y");
        assert_eq!(to_vec_sorted(g.ys_from_x(x)), vec![(y, &30)], "ys_from_x");
    }

    #[test]
    fn insert_edge_2() {
        let mut g = BipartiteGraph::new();
        let x = g.insert_x(10);
        let y = g.insert_y(20);
        g.insert_edge(x, y, 30);
        g.insert_edge(x, y, 40);
        assert_eq!(
            to_vec_sorted(g.edges()),
            vec![(x, y, &30), (x, y, &40)],
            "edges"
        );
        assert_eq!(
            to_vec_sorted(g.xs_from_y(y)),
            vec![(x, &30), (x, &40)],
            "xs_from_y"
        );
        assert_eq!(
            to_vec_sorted(g.ys_from_x(x)),
            vec![(y, &30), (y, &40)],
            "ys_from_x"
        );
    }

    #[test]
    fn remove_x() {
        let mut g = BipartiteGraph::new();
        let y = g.insert_y(0);
        let x = g.insert_x(1);
        g.insert_edge(x, y, 2);
        g.remove_x(x);
        assert_eq!(to_vec_sorted(g.xs()), vec![], "xs");
        assert_eq!(to_vec_sorted(g.ys()), vec![(y, &0)], "ys");
        assert_eq!(to_vec_sorted(g.edges()), vec![], "edges");
    }
}
