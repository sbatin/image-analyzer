use std::hash::Hash;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DisjointSet<T> {
    parents: Vec<usize>,
    values: HashMap<T, usize>,
}

fn find_parent(parents: &mut [usize], key: usize) -> usize {
    let mut k = key;
    let mut p = parents[key];

    while p != k {
        let pp = parents[p];
        parents[k] = pp;
        k = p;
        p = pp
    }

    p
}

impl<T: Eq + Hash> DisjointSet<T> {
    pub fn new() -> Self {
        DisjointSet {
            parents: Vec::new(),
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> usize {
        let key = self.values.len();
        self.parents.push(key);
        self.values.insert(value, key);
        key
    }

    pub fn find(&mut self, value: &T) -> usize {
        let key = self.values[value];
        find_parent(&mut self.parents, key)
    }

    pub fn union(&mut self, va: &T, vb: &T) {
        let pa = self.find(va);
        let pb = self.find(vb);

        if pa == pb {
            return;
        }

        self.parents[pa] = pb;
    }

    pub fn into_vec(self) -> Vec<Vec<T>> {
        let mut groups = HashMap::new();
        let mut parents = self.parents;

        for (v, k) in self.values.into_iter() {
            let p = find_parent(&mut parents, k);
            let vs = groups
                .entry(p)
                .or_insert(Vec::new());
            vs.push(v);
        }

        groups.into_values().collect()
    }
}
