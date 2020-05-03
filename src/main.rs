use num_traits::ops::wrapping::WrappingAdd;
use std::collections::{HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt::{Debug, Display};
use std::{fmt, fs};
use std::cmp::min;

// todo:
// figure out difference between u32 and usize
// find out if it's better to store array size or call len() each time
// implement my own (generic) Hasher to replace DefaultHasher
// use 0 as unused value and prevent 0 from being result of get_hash() - change i32 to usize
// test different neighborhood sizes and resize load factor thresholds - edit: no threshold needed!! resizes on failed insert

fn main() {
    //print!("{}", djb2_hash(&String::from("piodqwpowdqopmkwq"), 50));

    // let mut hasher = DefaultHasher::new();
    // "wesffessdweasdw".hash(&mut hasher);
    // print!("{}", hasher.finish() % 100)

    let mut words = Vec::new();

    let filename = "words.txt";
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    for line in contents.split("\n") {
        words.push(line)
    }

    let mut map: MyHashMap<u32, &str> = MyHashMap::new();
    for i in 0..words.len() {
        map = map.insert(i as u32, words[i]);
        print!("{}", map);
        println!();
        println!();
    }
    println!("{}", map);
    println!("got: {:?}", map.get(&2));
    println!("{} {}", "table size:", map.table.len());
    println!("{} {}", "academic hash:", get_hash(&"academic", 32, map.neighborhood_size));

    let _rust_map: HashMap<&str, ()> = HashMap::new();
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
    table: Vec<Entry<K, V>>, // stores tuples of keys, value, home_index
    neighborhood_size: usize
}

// trait Capacity {
//     const CAPACITY: u32;
// }
//
// impl<K, V> Capacity for MyHashMap<K, V> {
//     const CAPACITY: usize = 32;
// }

impl<K: Default + Hash + Copy + Clone + Debug + Eq + Display, V: Default + Hash + Copy + Clone + Debug + Eq + Display> MyHashMap<K, V> {
    pub fn new () -> Self {
        // should starting size be 32?
        // do i use *V or &V or V?
        Self { table: vec![Entry::new(); 16], neighborhood_size: 4}
    }

    pub fn new_param (initial_size: usize) -> Self {
        Self { table: vec![Entry::new(); initial_size], neighborhood_size: 16}
    }

    // either inserts new key or changes value of existing one
    pub fn insert (mut self, key: K, val: V) -> MyHashMap<K, V> {
        // should i store actual key and val or references??
        let home = get_hash(&key, self.table.len(), self.neighborhood_size);
        if self.table[home].home == 0 {
            self.table[home] = Entry { key, val, home };
            return self
        } else {
            // look through neighborhood
            for i in home..(min(home + self.neighborhood_size, self.table.len())) {
                // if empty space, store key/value pair there
                if self.table[i].home == 0 {
                    self.table[i] = Entry { key, val, home };
                    return self
                }
            }

            // if no room in neighborhood, look through the next neighborhood-sized interval
            // ... for a blank space that we could steal
            // todo: stop looking for empty space after a certain distance away from home? just go ahead and resize?
            for mut empty_i in home..self.table.len() { //should iterate until it finds first empty, then hand over to inner loop to shift to neighborhood
                if self.table[empty_i].home == 0 {
                    let mut starting_i = empty_i - (self.neighborhood_size - 1);
                    'inner: loop {
                        // found an empty space! now let's find something we can swap with it
                        for swap_cand_i in starting_i..(starting_i + self.neighborhood_size) {
                            // if we wouldn't be moving the swapped element too far from home...
                            if empty_i - self.table[swap_cand_i].home < self.neighborhood_size {
                                self.table[empty_i] = self.table[swap_cand_i];
                                self.table[swap_cand_i] = Entry { key, val, home }; //todo: make this empty?
                                // print_vec(&self.table);
                                empty_i = swap_cand_i; // empty space is now moved

                                //prepare to restart search for another swap
                                if empty_i as i32 - (self.neighborhood_size as i32 - 1) > 0 {
                                    starting_i = empty_i - (self.neighborhood_size - 1)
                                } else {
                                    starting_i = 0
                                }

                                if empty_i - home < self.neighborhood_size {
                                    // we are now within the neighborhood, so put new entry in empty space
                                    self.table[empty_i] = Entry { key, val, home };
                                    return self
                                } else {
                                    continue 'inner // look for another swap to move empty closer (or into) neighborhood
                                }
                            }
                        }
                        // can't swap anything with empty, need to resize
                        // a.k.a. if we're here, we failed to insert the new entry into the table
                        println!("{}", "im resizing (location 1)!");
                        self = self.expand();
                        return self.insert(key, val); // after resize, insert entry
                    }
                }
            }
            // can't find *any* empty slots -> need to resize
            println!("{}", "im resizing (location 2)!");
            println!("{} {}", "just failed to place:", Entry {key, val, home});
            self = self.expand();
            return self.insert(key, val); // after resize, insert entry
        }
        // shouldn't get here (?) - to satisfy compiler
        println!("{}", "oooops");
        return self
    }

    // removes a key/value pair from the map (soft delete not needed)
    pub fn remove (&mut self, key: &K) {
        let table_len = self.table.len();
        self.table[get_hash(key, table_len, self.neighborhood_size)] = Entry::new();
    }

    // returns the value corresponding to a key, or None if key not in table
    pub fn get (&self, key: &K) -> Option<V> {
        let home = get_hash(&key, self.table.len(), self.neighborhood_size);
        for i in home..(home + self.neighborhood_size) {
            let entry =self.table[i];
            if entry.home == home && entry.key.eq(key) {
                return Some(entry.val)
            }
        }
        return None
    }

    pub fn contains_key (&self, key: &K) -> bool {
        return self.get(key).is_none()
    }

    // number of elements in table
    pub fn len (&self) -> usize {
        self.table.len()
    }

    // called when load factor exceeds ~ 0.8 ?
    // doubles size of table and rehashes entries
    // need to assign old map to result
    fn expand (mut self) -> MyHashMap<K, V> {
        // should i allocate a new struct or just reassign the table vec?
        let new_table_size = self.table.len() * 2;
        println!("{} {}", "expanding to", new_table_size);
        //let mut new_table: Vec<Entry<K, V>> = vec![Entry::new(); new_table_len];
        let mut new_map: MyHashMap<K, V> = MyHashMap::new_param(new_table_size);
        for entry in self.table.as_slice() {
            if entry.home == 0 {  //unused space
                continue
            }
            //new_table[get_hash(&entry.key, new_table_len)] = *entry //nooooo sillly
            // need to call insert() somehow
            new_map = new_map.insert(entry.key, entry.val);
        }
        //self.table = new_table
        new_map
    }

    // // called when load factor falls below ~ 0.4 ?
    // // halves size of table and rehashes entries
    // fn shrink () {
    //
    // }
}

impl<K: Display, V: Display> fmt::Display for MyHashMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret_str: String = String::new();
        for (i, elem) in self.table.iter().enumerate() {
            let elem_str = format!("i: {}, k: {}, v: {}, h: {}\n", i, elem.key, elem.val, elem.home);
            ret_str = format!("{}{}", ret_str, elem_str)
        }
        write!(f, "{}", ret_str)
    }
}

impl<K: Display, V: Display> fmt::Display for Entry<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elem_string = format!("k: {}, v: {}, h: {}\n", self.key, self.val, self.home);
        write!(f, "{}", elem_string)
    }
}

// impl<T> Default for MyHashSet<T> {
//     fn default () -> MyHashSet<T> {
//         MyHashSet{array: [T; 1]}
//     }
// }

// todo: this *cannot* return 0 as that is being used as empty marker
// make sure this won't cluster around 0 if 0 maps to something like 1
fn get_hash<T: Hash> (key: &T, table_size: usize, neighborhood_size: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = (hasher.finish() % (table_size - neighborhood_size) as u64) as usize;
    if hash == 0 {
        return 1
    }
    return hash
}

fn print_vec<T: Display> (vec: &Vec<T>) {
    for elem in vec {
        print!("{}", elem)
    }
}

// // todo: make generic
// // returns djb2 hash of input mod arr_size
// fn djb2_hash (input: &str, arr_size: u32) -> u32 {
//     let mut hash: u32 = 5381;
//     for c in input.chars() {
//         hash = (hash << 5).wrapping_add(hash).wrapping_add(c as u32);
//         // println!("{}", c);
//         // println!("{}", hash);
//     }
//     return hash % arr_size
// }
//
// // from: https://stackoverflow.com/questions/25917260/getting-raw-bytes-from-packed-struct
// unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
//     ::std::slice::from_raw_parts(
//         (p as *const T) as *const u8,
//         ::std::mem::size_of::<T>(),
//     )
// }

// // returns djb2 hash of input mod arr_size
// fn djb2_hash<T> (input: &T, arr_size: u32) -> u32 {
//     let mut hash: u32 = 5381;
//     //let bytes: &[u8] = unsafe { any_as_u8_slice(input) };
//     //println!("{:?}", bytes);
//     for b in bytes {
//         hash = (hash << 5).wrapping_add(hash).wrapping_add(*b as u32);
//         println!("{}", b);
//         println!("{}", hash);
//     }
//     return hash % arr_size
// }
