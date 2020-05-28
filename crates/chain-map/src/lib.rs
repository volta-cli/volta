use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

#[derive(Default, Debug)]
pub struct ChainMap<K, V> {
    maps: Vec<HashMap<K, V>>,
}

impl<K, V> ChainMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        ChainMap { maps: Vec::new() }
    }

    pub fn push_map(&mut self, map: HashMap<K, V>) {
        self.maps.push(map)
    }

    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.maps.iter().any(|m| m.contains_key(k))
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.maps.iter().find_map(|m| m.get(k))
    }
}

impl<K, V> FromIterator<HashMap<K, V>> for ChainMap<K, V> {
    fn from_iter<I>(iter: I) -> ChainMap<K, V>
    where
        I: IntoIterator<Item = HashMap<K, V>>,
    {
        ChainMap {
            maps: Vec::from_iter(iter),
        }
    }
}
