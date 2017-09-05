/*
 *  Copyright 2017 Gianmarco Garrisi
 *
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

// an improvement in terms of complexity would be to use a bare HashMap
// as vec instead of the OrderMap
use ::iterators::*;

use std::cmp::{Ord, Eq};
use std::hash::Hash;
use std::borrow::Borrow;
use std::iter::Iterator;

use ordermap::OrderMap;

/// A priority queue with efficient change function to change the priority of an
/// element.
///
/// The priority is of type P, that must implement `std::cmp::Ord`.
///
/// The item is of type I, that must implement `Hash` and `Eq`.
///
/// Implemented as a heap of indexes, stores the items inside an `OrderMap`
/// to be able to retrieve them quickly.
#[derive(Clone, Default, Eq)]
pub struct PriorityQueue<I, P>
    where I: Hash+Eq,
          P: Ord {
    map: OrderMap<I, Option<P>>, // Stores the items and assign them an index
    heap: Vec<usize>,    // Implements the heap of indexes
    qp: Vec<usize>,      // Performs the translation from the index
    // of the map to the index of the heap
    size: usize          // The size of the heap
}

impl<I, P> PriorityQueue<I, P>
    where P: Ord,
          I: Hash + Eq {

    /// Creates an empty `PriorityQueue`
    pub fn new() -> PriorityQueue<I, P> {
        PriorityQueue{
            map: OrderMap::new(),
            heap: Vec::new(),
            qp: Vec::new(),
            size: 0
        }
    }

    /// Creates an empty `PriorityQueue` with the specified capacity.
    ///
    /// The internal collections will be able to hold at least `capacity`
    /// elements without reallocating.
    /// If `capacity` is 0, there will be no allocation.
    pub fn with_capacity(capacity: usize) -> PriorityQueue<I, P> {
        PriorityQueue{
            map: OrderMap::with_capacity(capacity),
            heap:     Vec::with_capacity(capacity),
            qp:       Vec::with_capacity(capacity),
            size: 0
        }
    }

    /// Returns an iterator in arbitrary order over the
    /// (item, priority) elements in the queue
    pub fn iter<'a>(&'a self) -> ::pqueue::Iter<'a, I, P>  {
        ::pqueue::Iter{iter: self.map.iter()}
    }

    /// Returns the couple (item, priority) with the greatest
    /// priority in the queue, or None if it is empty.
    ///
    /// Computes in **O(1)** time
    pub fn peek(&self) -> Option<(&I, &P)>{
        if self.size == 0 { return None }
        self.map.get_index(self.heap[0]).map(|(k, v)| (k, v.as_ref().unwrap()))
    }

    /// Returns the couple (item, priority) with the greatest
    /// priority in the queue, or None if it is empty.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    ///
    /// Computes in **O(1)** time
    pub fn peek_mut(&mut self) -> Option<(&mut I, &P)> {
        if self.size == 0 { return None }
        self.map.get_index_mut(self.heap[0])
            .map(|(k, v)| (k, v.as_ref().unwrap()))
    }

    /// Returns the number of elements the internal map can hold without
    /// reallocating.
    ///
    /// This number is a lower bound; the map might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    pub fn capacity(&self)->usize {
        self.map.capacity()
    }

    // reserve_exact -> OrderMap does not implement reserve_exact

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `PriorityQueue`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve(&mut self, additional: usize){
        self.map.reserve(additional);
        self.heap.reserve(additional);
        self.qp.reserve(additional);
    }

    /// Shrinks the capacity of the internal data structures
    /// that support this operation as much as possible.
    pub fn shrink_to_fit(&mut self){
        self.heap.shrink_to_fit();
        self.qp.shrink_to_fit();
    }

    /// Removes the item with the greatest priority from
    /// the priority queue and returns the pair (item, priority),
    /// or None if the queue is empty.
    pub fn pop(&mut self) -> Option<(I, P)> {
        if self.size == 0 {
            return None;
        }
        let result = self.swap_remove();
        if self.size > 0 {
            self.heapify(0);
        }
        result
    }

    /// Insert the item-priority pair into the queue.
    ///
    /// If an element equals to `item` was already into the queue,
    /// it is updated and the old value of its priority returned in `Some`;
    /// otherwise, return `None`.
    ///
    /// Computes in **O(log(N))** time.
    pub fn push(&mut self, item: I, priority: P) -> Option<P>{
        let mut pos;
        let oldp;
        if self.map.contains_key(&item){
            // FIXME: When the compiler get fixed,
            // write this part in a more efficient fashon
            {
                let (index, old_item, p) =
                    self.map.get_pair_index_mut(&item).unwrap();
                *old_item = item;
                oldp = p.take();
                *p = Some(priority);
                pos = self.qp[index];
            }
            let tmp = self.heap[pos];
            while (pos > 0) &&
                (self.map.get_index(self.heap[parent(pos)]).unwrap().1 <
                 self.map.get_index(self.heap[pos]).unwrap().1)
            {
                self.heap[pos] = self.heap[parent(pos)];
                self.qp[self.heap[pos]] = pos;
                pos = parent(pos);
            }
            self.heap[pos] = tmp;
            self.qp[tmp] = pos;
            self.heapify(pos);
            return oldp;
        }
        // insert the item, priority into the OrderMap
        self.map.insert(item, Some(priority)).map(|o| o.unwrap());
        // ... and get a reference to the priority
        let priority = self.map.get_index(self.size).unwrap().1;
        // copy the actual size of the heap
        let mut i = self.size;
        let k = i;
        // add the new element in the qp vector as the last in the heap
        self.qp.push(i);
        self.heap.push(0);
        // from the leaf go up to root or until an element with priority greater
        // than the new element is found
        while (i > 0) &&
            ( self.map.get_index(self.heap[parent(i)]).unwrap().1 < &priority ){
                self.heap[i] = self.heap[parent(i)];
                self.qp[self.heap[i]] = i;
                i = parent(i);
            }
        // put the new element into the heap and
        // update the qp translation table and the size
        self.heap[i] = k;
        self.qp[k] = i;
        self.size += 1;
        None
        //}
    }

    /// Change the priority of an Item returning the old value of priority,
    /// or `None` if the item wasn't in the queue.
    ///
    /// The item is found in **O(1)** thanks to the hash table.
    /// The operation is performed in **O(log(N))** time.
    pub fn change_priority<Q: ?Sized>(&mut self, item: &Q, new_priority: P)
                                      -> Option<P>
        where I: Borrow<Q>,
              Q:Eq + Hash
    {
        let mut pos = 0;
        let r =
            if let Some((index, _, p))= self.map.get_pair_index_mut(item) {
                let oldp = p.take();
                *p = Some(new_priority);
                pos = self.qp[index];
                oldp
            } else {
                None
            };
        if r.is_some() {
            let tmp = self.heap[pos];
            while (pos > 0) &&
                (self.map.get_index(self.heap[parent(pos)]).unwrap().1 <
                 self.map.get_index(self.heap[pos]).unwrap().1)
            {
                self.heap[pos] = self.heap[parent(pos)];
                self.qp[self.heap[pos]] = pos;
                pos = parent(pos);
            }
            self.heap[pos] = tmp;
            self.qp[tmp] = pos;
            self.heapify(pos);
        }
        r
    }

    /// Change the priority of an Item using the provided function.
    /// The item is found in **O(1)** thanks to the hash table.
    /// The operation is performed in **O(log(N))** time (worst case).
    pub fn change_priority_by<Q: ?Sized, F>
        (&mut self, item: &Q, priority_setter: F)
        where I: Borrow<Q>,
              Q: Eq + Hash,
              F: FnOnce(P) -> P
    {
        let mut pos = 0;
        let mut found = false;
        if let Some((index, _, p))= self.map.get_pair_index_mut(item) {
            let oldp = p.take().unwrap();
            *p = Some(priority_setter(oldp));
            pos = self.qp[index];
            found = true;
        }
        if found {
            let tmp = self.heap[pos];
            while (pos > 0) &&
                (self.map.get_index(self.heap[parent(pos)]).unwrap().1 <
                 self.map.get_index(self.heap[pos]).unwrap().1)
            {
                self.heap[pos] = self.heap[parent(pos)];
                self.qp[self.heap[pos]] = pos;
                pos = parent(pos);
            }
            self.heap[pos] = tmp;
            self.qp[tmp] = pos;
            self.heapify(pos);
        }
    }

    /// Get the priority of an item, or `None`, if the item is not in the queue
    pub fn get_priority<Q: ?Sized>(&self, item: &Q) -> Option<&P>
        where I: Borrow<Q>,
              Q: Eq + Hash
    {
        self.map.get(item).map(|o| o.as_ref().unwrap())
    }

    /// Get the couple (item, priority) of an arbitrary element, as reference
    /// or `None` if the item is not in the queue.
    pub fn get<Q>(&self, item: &Q) -> Option<(&I, &P)>
        where I: Borrow<Q>,
              Q: Eq + Hash
    {
        self.map.get_pair(item).map(|(k, v)| (k, v.as_ref().unwrap()))
    }

    /// Get the couple (item, priority) of an arbitrary element, or `None`
    /// if the item was not in the queue.
    ///
    /// The item is a mutable reference, but it's a logic error to modify it
    /// in a way that change the result of  `Hash` or `Eq`.
    ///
    /// The priority cannot be modified with a call to this function.
    /// To modify the priority use `push`, `change_priority` or
    /// `change_priority_by`.
    pub fn get_mut<Q>(&mut self, item: &Q) -> Option<(&mut I, &P)>
        where I: Borrow<Q>,
              Q: Eq + Hash
    {
        self.map.get_pair_mut(item).map(|(k, v)| (k, v.as_ref().unwrap()))
    }

    /// Returns the items not ordered
    pub fn into_vec(self) -> Vec<I> {
        self.map.into_iter().map(|(i, _)| i).collect()
    }

    /// Implements an HeapSort
    pub fn into_sorted_vec(mut self) -> Vec<I> {
        let mut res = Vec::with_capacity(self.size);
        while let Some((i, _)) = self.pop() {
            res.push(i);
        }
        res
    }

    /// Returns the number of elements in the priority queue.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns true if the priority queue contains no elements.
    pub fn is_empty(&self) -> bool {
        self.size==0
    }

    /// Drops all items from the priority queue
    pub fn clear(&mut self){
        self.heap.clear();
        self.qp.clear();
        self.map.clear();
        self.size=0;
    }

    /// Move all items of the `other` queue to `self`
    /// ignoring the items Eq to elements already in `self`
    /// At the end, `other` will be empty.
    ///
    /// **Note** that at the end, the priority of the duplicated elements
    /// inside self may be the one of the elements in other,
    /// if other is longer than self
    pub fn append(&mut self, other: &mut Self) {
        if other.size > self.size {
            ::std::mem::swap(self, other);
        }
        if other.size == 0 {
            return;
        }
        let drain = other.map.drain(..);
        // what should we do for duplicated keys?
        // ignore
        for (k, v) in drain {
            if !self.map.contains_key(&k) {
                let i = self.size;
                self.map.insert(k, v);
                self.heap.push(i);
                self.qp.push(i);
                self.size += 1;
            }
        }
        other.heap.clear();
        other.qp.clear();
        self.heap_build();
    }

    /// Generates a new iterator from self that
    /// will extract the elements from the one with the highest priority
    /// to the lowest one.
    pub fn into_sorted_iter(self) -> IntoSortedIter<I, P> {
        IntoSortedIter{pq: self}
    }
    /**************************************************************************/
    /*                            internal functions                          */


    /// Remove and return the element with the max priority
    /// and swap it with the last element keeping a consistent
    /// state.
    /// Computes in **O(1)** time (average)
    fn swap_remove(&mut self) -> Option<(I, P)>{
        // swap_remove the head
        let head = self.heap.swap_remove(0);
        self.size -= 1;
        // swap remove the old heap from the qp
        if self.size == 0 {
            self.qp.pop();
            return self.map.swap_remove_index(head)
                .map(|(i, o)| (i, o.unwrap()));
        }
        self.qp[self.heap[0]] = 0;
        self.qp.swap_remove(head);
        if head < self.size {
            self.heap[self.qp[head]] = head;
        }
        // swap remove from the map and return to the client
        self.map.swap_remove_index(head)
            .map(|(i, o)| (i, o.unwrap()))
    }

    /// Swap two elements keeping a consistent state.
    ///
    /// Computes in **O(1)** time (average)
    fn swap(&mut self, a: usize, b:usize) {
        let (i, j) = (self.heap[a], self.heap[b]);
        self.heap.swap(a, b);
        self.qp.swap(i, j);
    }

    /// Internal function that restore the functional property of the heap
    fn heapify(&mut self, i: usize) {
        let (mut l, mut r) = (left(i), right(i));
        let mut i = i;
        let mut largest;
        if l < self.size &&
            self.map.get_index(self.heap[l]).unwrap().1 >
            self.map.get_index(self.heap[i]).unwrap().1
        {
            largest = l;
        } else {
            largest = i;
        }
        if r < self.size &&
            self.map.get_index(self.heap[r]).unwrap().1 >
            self.map.get_index(self.heap[largest]).unwrap().1
        {
            largest = r;
        }
        while largest != i {
            self.swap(i, largest);

            i = largest;
            l = left(i);
            r = right(i);
            if l < self.size &&
                self.map.get_index(self.heap[l]).unwrap().1 >
                self.map.get_index(self.heap[i]).unwrap().1
            {
                largest = l;
            }
            else {
                largest = i;
            }
            if r < self.size &&
                self.map.get_index(self.heap[r]).unwrap().1 >
                self.map.get_index(self.heap[largest]).unwrap().1
            {
                largest = r;
            }
        }
    }

    /// Internal function that transform the `heap`
    /// vector in a heap with its properties
    fn heap_build(&mut self){
        for i in (0..parent(self.size)).rev(){
            self.heapify(i);
        }
    }
}


//FIXME: fails when the vector contains repeated items
// FIXED: repeated items ignored
impl<I, P> From<Vec<(I, P)>> for PriorityQueue<I, P>
    where I: Hash+Eq,
          P: Ord {
    fn from(vec: Vec<(I, P)>) -> PriorityQueue<I, P>{
        let mut pq = PriorityQueue::with_capacity(vec.len());
        let mut i=0;
        for (item, priority) in vec {
            if !pq.map.contains_key(&item) {
                pq.map.insert(item, Some(priority));
                pq.qp.push(i);
                pq.heap.push(i);
                i+=1;
            }
        }
        pq.size=i;
        pq.heap_build();
        pq
    }
}

//FIXME: fails when the iterator contains repeated items
// FIXED: the item inside the pq is updated
// so there are two functions with different behaviours.
impl<I, P> ::std::iter::FromIterator<(I, P)> for PriorityQueue<I, P>
    where I: Hash+Eq,
          P: Ord {
    fn from_iter<IT>(iter: IT) -> PriorityQueue<I, P>
        where IT: IntoIterator<Item = (I, P)>{
        let iter = iter.into_iter();
        let (min, max) = iter.size_hint();
        let mut pq =
            if let Some(max) = max {
                PriorityQueue::with_capacity(max)
            } else if min != 0 {
                PriorityQueue::with_capacity(min)
            } else {
                PriorityQueue::new()
            };
        for (item, priority) in iter {
            if !pq.map.contains_key(&item){
                pq.map.insert(item, Some(priority));
                pq.qp.push(pq.size);
                pq.heap.push(pq.size);
                pq.size+=1;
            } else {
                let (old_item, old_priority) =
                    pq.map.get_pair_mut(&item).unwrap();
                *old_item = item;
                *old_priority = Some(priority);
            }
        }
        pq.heap_build();
        pq
    }
}

impl<I, P> ::std::iter::IntoIterator for PriorityQueue<I, P>
    where I: Hash+Eq,
          P: Ord {
    type Item = (I, P);
    type IntoIter = ::pqueue::IntoIter<I, P>;
    fn into_iter(self) -> ::pqueue::IntoIter<I, P> {
        ::pqueue::IntoIter{ iter: self.map.into_iter() }
    }
}

impl<I, P>  ::std::iter::Extend<(I, P)> for PriorityQueue <I, P>
    where I: Hash+Eq,
          P: Ord {
    fn extend <T: IntoIterator<Item=(I, P)>> (&mut self, iter: T) {
        let iter = iter.into_iter();
        let (min, max) = iter.size_hint();
        let mut rebuild = false;
        if let Some(max) = max {
            self.reserve(max);
            rebuild = better_to_rebuild(self.size, max);
        } else if min != 0 {
            self.reserve(min);
            rebuild = better_to_rebuild(self.size, min);
        }
        if rebuild {
            for (item, priority) in iter {
                if !self.map.contains_key(&item){
                    self.map.insert(item, Some(priority));
                    self.qp.push(self.size);
                    self.heap.push(self.size);
                    self.size+=1;
                } else {
                    let (old_item, old_priority) =
                        self.map.get_pair_mut(&item).unwrap();
                    *old_item = item;
                    *old_priority = Some(priority);
                }
            }
            self.heap_build();
        } else {
            for (item, priority) in iter {
                self.push(item, priority);
            }
        }
    }
}

use std::fmt;
impl<I, P> fmt::Debug for PriorityQueue<I, P>
    where I: fmt::Debug + Hash + Eq,
          P: fmt::Debug + Ord {
    fn  fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(self.heap.iter()
                     .map(|&i| self.map.get_index(i).unwrap())
                     .map(|(i, op)| (i, op.as_ref().unwrap())))
            .finish()
    }
}

use std::cmp::PartialEq;
impl<I, P1, P2> PartialEq<PriorityQueue<I, P2>> for PriorityQueue<I, P1> 
    where I: Hash+Eq,
          P1: Ord,
          P1: PartialEq<P2>,
Option<P1>: PartialEq<Option<P2>>,
          P2: Ord {
    
    fn eq(&self, other: &PriorityQueue<I, P2>) -> bool {
        self.map == other.map
    }
}

#[inline(always)]
/// Compute the index of the left child of an item from its index
fn left(i:usize) -> usize {
    (i*2) +1
}
#[inline(always)]
/// Compute the index of the right child of an item from its index
fn right(i:usize) -> usize {
    (i*2) +2
}
#[inline(always)]
/// Compute the index of the parent element in the heap from its index
fn parent(i:usize) -> usize{
    (i-1) /2
}

#[inline(always)]
fn log2_fast(x: usize) -> usize {
    use std::mem::size_of;
    8 * size_of::<usize>() - (x.leading_zeros() as usize) - 1
}

// `rebuild` takes O(len1 + len2) operations
// and about 2 * (len1 + len2) comparisons in the worst case
// while `extend` takes O(len2 * log_2(len1)) operations
// and about 1 * len2 * log_2(len1) comparisons in the worst case,
// assuming len1 >= len2.
#[inline]
fn better_to_rebuild(len1: usize, len2: usize) -> bool {
    2 * (len1 + len2) < len2 * log2_fast(len1)
}

#[cfg(serde)]
mod serde {
    extern crate serde;
    use serde::ser::{Serialize, Serializer, SerializeMap};
    impl<I, P> Serialize for PriorityQueue<I, P>
        where I: Hash + Eq + Serialize,
              P: Ord + Serialize {
        fn serialize<T> (&self, serializer: S) -> Result<S::Ok, S::Error>
            where S:Serializer {
            let mut map_serializer = serializer.serialize_map(Some(self.size))?;
            for (k, v) in self.map {
                map_serializer.serialize_key(k)?;
                map_serializer.serialize_value(v)?;
            }
            map_serializer.end()
        }
    }

    use serde::de::{Deserialize, Deserializer, Visitor, Error};
    impl<'de, I, P> Deserialize<'de> for PriorityQueue<I, P>
        where I: Hash + Eq + Deserialize<'de>,
              P: Ord + Deserialize<'de> {
        fn deserialize<D>(deserializer: D) -> Result<PriorityQueue<I, P>, D::Error>
            where D: Deserializer<'de> {
            let pq = deserializer.deserialize_map(PQVisitor<K, V>{})?;
            pq.heap_build();
            Ok(pq)
        }
    }

    struct PQVisitor<I, P>;
    impl<'de, I, P> Visitor<'de> for PQVisitor<I, P>
        where I: Hash + Eq + Deserialize<'de>,
              P: Ord + Deserialize<'de> {
        type Value = PriorityQueue<I, P>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "A priority queue")
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(PriorityQueue::new())
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de>{
            let mut pq: PriorityQueue<I, P> = 
                if let Some(size) = map.size_hint() {
                    PriorityQueue::with_capacity(size);
                } else {
                    PriorityQueue::new();
                };

            while let Some((item, priority)) = map.next_entry()? {
                pq.map.insert(item, Some(priority));
                pq.qp.push(self.size);
                pq.heap.push(self.size);
                pq.size+=1;
            }
            pq.heap_build();
            // if it is guaranteed that deserialization follow the same order of
            // serialization, heap_build is useless, but anyway should be O(n)
        }
    }
}
