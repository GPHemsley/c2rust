use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;

struct Scope<T> {
    name_map: HashMap<T, String>,
    used: HashSet<String>,
}

impl<T: Clone + Eq + Hash> Scope<T> {
    pub fn new() -> Self {
        Self::new_with_reserved(HashSet::new())
    }

    pub fn new_with_reserved(reserved: HashSet<String>) -> Self {
        Scope {
            name_map: HashMap::new(),
            used: reserved,
        }
    }

    pub fn contains_key(&self, key: &T) -> bool {
        self.name_map.contains_key(key)
    }

    pub fn contains_value(&self, val: &str) -> bool {
        self.used.contains(val)
    }

    pub fn reserve(&mut self, val: String) {
        self.used.insert(val);
    }
}

pub struct Renamer<T> {
    scopes: Vec<Scope<T>>,
    next_fresh: u64,
}

impl<T: Clone + Eq + Hash> Renamer<T> {

    /// Creates a new renaming environment with a single, empty scope. The given set of
    /// reserved names will exclude those names from being chosen as the mangled names from
    /// the insert method.
    pub fn new(reserved_names: HashSet<String>) -> Self {
        Renamer {
            scopes: vec![Scope::new_with_reserved(reserved_names)],
            next_fresh: 0,
        }
    }

    /// Introduces a new name binding scope
    pub fn add_scope(&mut self) {
        self.scopes.push(Scope::new())
    }

    /// Drops the current name binding scope
    pub fn drop_scope(&mut self) {
        if self.scopes.len() == 1 {
            panic!("Attempting to drop outermost scope")
        }

        self.scopes.pop();
    }

    fn current_scope(&self) -> &Scope<T> {
        self.scopes.last().expect("Expected a scope")
    }

    fn current_scope_mut(&mut self) -> &mut Scope<T> {
        self.scopes.last_mut().expect("Expected a scope")
    }

    /// Is the mangled name currently in use
    fn is_target_used(&self, key: &str) -> bool {
        let key = key.to_string();

        self.scopes.iter().any(|x| x.contains_value(&key))
    }

    fn pick_name (&mut self, basename: &str) -> String {

        let mut target = basename.to_string();
        for i in 0.. {
            if self.is_target_used(&target) {
                target = format!("{}_{}", basename, i);
            } else {
                break
            }
        }

        self.current_scope_mut().reserve(target.clone());

        target
    }

    /// Introduce a new name binding into the current scope. If the key is unbound in
    /// the current scope then Some of the resulting mangled name is returned, otherwise
    /// None.
    pub fn insert(&mut self, key: T, basename: &str) -> Option<String> {

        if self.current_scope().contains_key(&key) {
            return None
        }

        let target = self.pick_name(basename);

        self.current_scope_mut().name_map.insert(key, target.clone());

        Some(target)
    }

    /// Lookup the given key in all of the scopes returning Some of the matched mangled name
    /// if one exists, otherwise None.
    pub fn get(&self, key: &T) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(target) = scope.name_map.get(key) {
                return Some(target.to_string())
            }
        }
        None
    }

    pub fn fresh(&mut self) -> String {
        let fresh = self.next_fresh;
        self.next_fresh += 1;
        self.pick_name(&format!("fresh{}", fresh))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let keywords = vec!["reserved"].into_iter().map(str::to_string).collect();
        let mut renamer = Renamer::new(keywords);

        let one1 = renamer.insert(1,"one").unwrap();
        let one2 = renamer.get(&1).unwrap();
        assert_eq!(one1, one2);

        let reserved1 = renamer.insert(2, "reserved").unwrap();
        let reserved2 = renamer.get(&2).unwrap();
        assert_eq!(reserved1, "reserved_0");
        assert_eq!(reserved2, "reserved_0");
    }

    #[test]
    fn scoped() {
        let mut renamer = Renamer::new(HashSet::new());

        let one1 = renamer.insert(10, "one").unwrap();
        renamer.add_scope();

        let one2 = renamer.get(&10).unwrap();
        assert_eq!(one1, one2);

        let one3 = renamer.insert(20,"one").unwrap();
        let one4 = renamer.get(&20).unwrap();
        assert_eq!(one3, one4);
        assert_ne!(one3, one2);

        renamer.drop_scope();

        let one5 = renamer.get(&10).unwrap();
        assert_eq!(one5, one2);
    }

    #[test]
    fn forgets() {
        let mut renamer = Renamer::new(HashSet::new());
        assert_eq!(renamer.get(&1), None);
        renamer.add_scope();
        renamer.insert(1,"example");
        renamer.drop_scope();
        assert_eq!(renamer.get(&1), None);
    }
}
