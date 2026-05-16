#[derive(Debug, PartialEq)]
pub struct ArrayMap<K: Eq, V>(pub(crate) Vec<(K, V)>);

impl<K: Eq, V> ArrayMap<K, V> {
	pub(crate) fn new() -> Self {
		Self(Vec::new())
	}

	pub(crate) fn len(&self) -> usize {
		self.0.len()
	}

	pub(crate) fn get(&self, key: &K) -> Option<&V> {
		for (k, v) in &self.0 {
			if k == key {
				return Some(&v);
			}
		}
		return None;
	}

	pub(crate) fn insert(&mut self, key: K, val: V) {
		for (k, v) in self.0.iter_mut() {
			if *k == key {
				*v = val;
				return;
			}
		}
		self.0.push((key, val));
	}

	pub(crate) fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
		self.0.iter()
	}
}

impl<K: Eq, V> std::ops::Index<&K> for ArrayMap<K, V> {
	type Output = V;

	fn index(&self, key: &K) -> &V {
		self.get(key).expect("no entry found for key")
	}
}

impl<K: Eq, V> IntoIterator for ArrayMap<K, V> {
	type Item = (K, V);
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
