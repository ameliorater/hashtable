use num_traits::ops::wrapping::WrappingAdd;
use std::collections::{HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher, BuildHasherDefault};
use std::fmt::{Debug, Display};
use std::{fmt};
use std::cmp::min;
use std::time::SystemTime;

// todo:
// find out if it's better to store array size or call len() each time
// implement my own (generic) Hasher to replace DefaultHasher
// better way to prevent hashing to 0
// test different neighborhood sizes (see what load factors they result in)
// implement keys(), shrink(), etc.

fn main() {
    // let mut words = Vec::new();

    // // SMALL INPUT
    // let filename = "all-words.txt";
    // let contents = fs::read_to_string(filename)
    //     .expect("Something went wrong reading the file");
    // for line in contents.split("\n") {
    //     words.push(line)
    // }
    // println!("{}", words.len());

    // // LARGE INPUT
    // let filename = "sampleFICT.txt";
    // let contents = fs::read_to_string(filename)
    //     .expect("Something went wrong reading the file");
    // for line in contents.split(" ") {
    //     words.push(line)
    // }
    // println!("contains: {}", words.len());

    let start_time = SystemTime::now();

    // // STRING
    // // RUST BUILT-IN
    // // let mut rust_map: HashMap<u32, &str> = HashMap::new();
    // // for i in 0..words.len() {
    // //     rust_map.insert(i as u32, words[i]);
    // // }
    //
    // // MY TABLE
    // let mut map: MyHashMap<u32, &str> = MyHashMap::new();
    // for i in 0..words.len() {
    //     map.insert(i as u32, words[i]);
    //     //print!("{}\n\n", map);
    // }

    // INT
    let size = 1e6 as usize;
    println!("input size: {}", size);

    // // RUST BUILT-IN
    // let mut rust_map: HashMap<usize, ()> = HashMap::new();
    // for i in 0..size {
    //     rust_map.insert(i, ());
    // }
    // // for i in 0..size {
    // //     rust_map.get(&i);
    // // }
    // // println!("{}", rust_map);
    // println!("{} {}", "final table size:", rust_map.capacity());
    // println!("{} {}", "final entries stored:", rust_map.len());

    // MY TABLE
    let mut map: MyHashMap<usize, usize> = MyHashMap::new();
    for i in 0..size {
        map.insert(i, i);
    }
    // for i in 0..size {
    //     map.get(&i);
    // }
    println!("{} {}", "final table size:", map.table.len());
    println!("{} {}", "final entries stored:", map.len());

    println!("elapsed time: {:?}", SystemTime::now().duration_since(start_time).unwrap());
}

#[derive(Debug, Clone, Copy)]
struct Entry<K, V> {
    key: K,
    val: V,
    home: usize
}

impl<K: Default + Clone, V: Default + Clone> Entry<K, V> {
    pub fn new () -> Self { Self { key: K::default(), val: V::default(), home: 0 } }
}

#[derive(Debug)]
struct MyHashMap<K, V> {
    table: Vec<Entry<K, V>>,
    H: usize  // neighborhood size
}

impl<K: Default + Hash + Copy + Clone + Eq + Display, V: Default + Hash + Copy + Clone + Eq + Display> MyHashMap<K, V> {
    pub fn new () -> Self {
        Self { table: vec![Entry::new(); 64], H: 32}
    }

    pub fn new_param (initial_size: usize) -> Self {
        Self { table: vec![Entry::new(); initial_size], H: 16}
    }

    // either inserts new key or changes value of existing one
    pub fn insert (&mut self, key: K, val: V) {
        let home = get_hash(&key, self.table.len(), self.H);
        // look through neighborhood for empty space
        let mut empty = 0; // (nothing will be stored at position 0)
        let mut same = 0;
        for i in home..(min(home + self.H, self.table.len())) {
            // find first empty space
            if empty == 0 && self.table[i].home == 0 {
                empty = i;
            }
            // if key is already in table...
            if self.table[i].home == home && self.table[i].key.eq(&key) {
                same = i;
                break
            }
        }

        // replace entry if it already existed, or store in empty space if one was found
        if same != 0 {
            self.table[same] = Entry { key, val, home };
            return
        } else if empty != 0 {
            self.table[empty] = Entry { key, val, home };
            return
        }

        // if no room in neighborhood, look through the rest of the table for an empty space to swap with
        // ei -> (potentially) empty index, si -> interval starting index, ci -> swap candidate index
        for mut ei in home..self.table.len() {
            if self.table[ei].home == 0 {
                let mut si = ei - (self.H - 1);
                'inner: loop {
                    // found an empty space! now let's find something we can swap with it
                    for ci in si..(si + self.H) {
                        // if we wouldn't be moving the swapped element outside of its neighborhood
                        if ei - self.table[ci].home < self.H {
                            self.table[ei] = self.table[ci];
                            self.table[ci] = Entry { key, val, home };
                            ei = ci; // empty space is now moved

                            //prepare to restart search for another swap
                            if ei as i32 - (self.H as i32 - 1) > 0 {
                                si = ei - (self.H - 1)
                            } else {
                                si = 0
                            }

                            if ei - home < self.H {
                                // we are now within the neighborhood, so put new entry in empty space
                                self.table[ei] = Entry { key, val, home };
                                return
                            } else {
                                // look for another swap to move empty closer (or into) neighborhood
                                continue 'inner
                            }
                        }
                    }
                    // can't swap anything with empty space, need to resize
                    self.expand();
                    self.insert(key, val); // after resize, insert entry
                }
            }
        }
        // can't find any empty spaces, need to resize
        self.expand();
        self.insert(key, val); // after resize, insert entry
    }

    // removes a key/value pair from the map
    pub fn remove (&mut self, key: &K) -> Option<V> {
        self.find(key, true)  // finds and removes entry
    }

    // returns the value corresponding to a key, or None if key not in table
    pub fn get (&mut self, key: &K) -> Option<V> {
        self.find(key, false)  // finds and returns entry
    }

    // used for get and remove methods (to avoid duplicated code)
    fn find (&mut self, key: &K, remove: bool) -> Option<V> {
        let home = get_hash(&key, self.table.len(), self.H);
        for i in home..(home + self.H) {
            let entry = self.table[i];
            if entry.home == home && entry.key.eq(key) {
                if remove { self.table[i] = Entry::new() }
                return Some(entry.val)
            }
        }
        return None
    }

    pub fn contains_key (&mut self, key: &K) -> bool {
        return self.get(key).is_none()
    }

    // number of elements in table (not including empty spaces)
    pub fn len (&self) -> usize {
        return (&self.table).iter().filter((|e| { e.home != 0 })).count()
    }

    fn expand (&mut self) {
        let mut new_map: MyHashMap<K, V> = MyHashMap::new_param(self.table.len() * 2);
        for entry in self.table.as_slice() {
            if entry.home == 0 {  //unused space
                continue
            }
            new_map.insert(entry.key, entry.val);
        }
        *self = new_map
    }
}

impl<K: Display, V: Display> fmt::Display for MyHashMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret_str: String = String::new();
        for (i, elem) in self.table.iter().enumerate() {
            let elem_str = format!("i: {}, {}\n", i, elem);
            ret_str = format!("{}{}", ret_str, elem_str)
        }
        write!(f, "{}", ret_str)
    }
}

impl<K: Display, V: Display> fmt::Display for Entry<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "k: {}, v: {}, h: {}", self.key, self.val, self.home)
    }
}

fn get_hash<T: Hash> (key: &T, table_size: usize, neighborhood_size: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = (hasher.finish() % (table_size - (neighborhood_size - 1)) as u64) as usize;
    if hash == 0 { return 1 }  //because 0 is used as empty marker
    return hash
}

// currently unused

#[derive(Debug, Clone)]
pub struct MyHasher {
    hash: usize,
}

impl MyHasher {
    pub fn new () -> MyHasher {
        MyHasher { hash: 0 }
    }
}

impl Hasher for MyHasher {
    fn finish(&self) -> u64 {
        // Your hashing algorithm goes here!
        self.hash as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        // Your hashing algorithm goes here!
        let mut hash: usize = 5381;
        for b in bytes {
            hash = (hash << 5).wrapping_add(hash).wrapping_add(*b as usize);
        }
        self.hash = hash
    }
}