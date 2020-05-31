// Copyright (c) 2020 by Enrique Pérez Arnaud <enrique at cazalla.net>    
//    
// This file is part of the modus_ponens project.    
// http://www.modus_ponens.net    
//    
// The modus_ponens project is free software: you can redistribute it and/or modify    
// it under the terms of the GNU General Public License as published by    
// the Free Software Foundation, either version 3 of the License, or    
// (at your option) any later version.    
//    
// The modus_ponens project is distributed in the hope that it will be useful,    
// but WITHOUT ANY WARRANTY; without even the implied warranty of    
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the    
// GNU General Public License for more details.    
//    
// You should have received a copy of the GNU General Public License    
// along with any part of the modus_ponens project.    
// If not, see <http://www.gnu.org/licenses/>.

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::mem;

use crate::segment::MPSegment;
use crate::matching::{ MPMatching, get_or_key, get_or_key_owning };

#[derive(Debug, Clone)]
pub struct MPPath<'a> {
    pub value: &'a MPSegment,
    pub segments: Vec<&'a MPSegment>,
    identity: u64,
}

impl<'a> MPPath<'a> {
    pub fn new(segments: Vec<&'a MPSegment>) -> MPPath {
        let value = *segments.last().expect("no empty paths");
        let mut hasher = DefaultHasher::new();
        for &segment in segments.iter() {
            segment.name.hash(&mut hasher);
        }
        value.text.hash(&mut hasher);
        let identity = hasher.finish();
        MPPath { value, segments, identity }
    }
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    pub fn starts_with(&self, path: &MPPath) -> bool {
        let lself = self.len();
        let lpath = path.len();
        lself >= lpath && &self.segments[0..lpath] == &path.segments[0..lpath]
    }
    pub fn starts_with_slice(&self, path_slice: &'a [&'a MPSegment]) -> bool {
        let lself = self.len();
        let lpath = path_slice.len();
        lself >= lpath && self.segments[0..lpath] == path_slice[0..lpath]
    }
    pub fn sub_path(&'a self, lpath: usize) -> MPPath<'a> {
        let new_segments = &self.segments[0..lpath];
        MPPath::new(new_segments.to_vec())
    }
    pub fn sub_slice(&'a self, lpath: usize) -> (&'a [&'a MPSegment], &'a MPSegment) {
        let segments = &self.segments[0..lpath];
        (segments, segments.last().expect("no empty paths"))
    }

    pub fn paths_after(&'a self, paths: &'a [MPPath]) -> usize {
        let mut seen = false;
        let mut path_starts_with_self: bool;
        let mut i = 0;
        for path in paths {
            if path.value.is_empty {
                i += 1;
                continue;
            }
            path_starts_with_self = path.starts_with(&self);
            if path_starts_with_self {
                seen = true;
            } else if seen {
                break;
            }
            i += 1;
        }
        i as usize
    }


    pub fn paths_after_slice(path_slice: &'a [&'a MPSegment], paths: &'a [MPPath<'a>]) -> &'a [MPPath<'a>] {
        let mut i: u16 = 0;
        for path in paths {
            if path.value.is_empty || !path.value.is_leaf {
                i += 1;
                continue;
            }
            if !path.starts_with_slice(path_slice) {
                break;
            }
            i += 1;
        }
        &paths[i as usize..]
    }

    pub fn substitute(&'a self, matching: &'a MPMatching) -> MPPath {
        let mut new_segments = Vec::with_capacity(self.segments.len());
        for segment in self.segments.iter() {
            let new_segment = get_or_key(&matching, &segment);
            new_segments.push(new_segment);
            if &new_segment != segment {
                break;
            }
        }
        MPPath::new(new_segments)
    }

    pub fn substitute_to_string(&'a self, matching: &'a MPMatching) -> (&str, bool, Option<MPPath>) {
        let mut last_text = "";
        let mut is_leaf = false;
        let mut old_segments = Vec::with_capacity(self.segments.len());
        let mut is_new = false;
        for segment in self.segments.iter() {
            let new_segment = get_or_key(&matching, &segment);
            is_new = &new_segment != segment;
            last_text = new_segment.text.as_str();
            is_leaf = new_segment.is_leaf;
            old_segments.push(*segment);
            if is_new {
                break;
            }
        }
        if is_new {
            let old_path = MPPath::new(old_segments);
            (last_text, is_leaf, Some(old_path))
        } else {
            (last_text, is_leaf, None)
        }
    }

    pub fn substitute_owning(&'a self, matching: MPMatching<'a>) -> (MPPath, Option<MPPath>) {
        let mut new_segments = Vec::with_capacity(self.segments.len());
        let mut old_segments = Vec::with_capacity(self.segments.len());
        let mut is_new = false;
        for segment in self.segments.iter() {
            let new_segment = get_or_key_owning(matching.clone(), &segment);
            is_new = &new_segment != segment;
            new_segments.push(new_segment);
            old_segments.push(*segment);
            if is_new {
                break;
            }
        }
        if is_new {
            new_segments.shrink_to_fit();
            old_segments.shrink_to_fit();
            let new_path = MPPath::new(new_segments);
            let old_path = MPPath::new(old_segments);
            (new_path, Some(old_path))
        } else {
            (MPPath::new(new_segments), None)
        }
    }

    pub fn substitute_paths_to_string(paths: Vec<MPPath<'a>>, matching: MPMatching<'a>) -> (String, MPMatching<'a>) {
        let mut fact = String::with_capacity(paths.len());
        let mut old_paths:Vec<MPPath> = Vec::with_capacity(paths.len());
        for path in paths.iter() {
            let mut seen = false;
            for opath in old_paths.iter() {
                if path.len() > opath.len() && path.starts_with(opath) {
                    seen = true;
                    break;
                }
            }
            if !seen {
                let (new_text, is_leaf, old_path) = path.substitute_to_string(&matching);
                if old_path.is_some() {
                    old_paths.push(old_path.unwrap());
                    fact.push_str(new_text);
                } else if is_leaf {
                    fact.push_str(new_text);
                }
            }
        }
        (fact, matching)
    }

    pub fn substitute_paths_owning(paths: Vec<MPPath<'a>>, matching: MPMatching<'a>) -> Vec<MPPath<'a>> {
        let mut new_paths: Vec<MPPath> = Vec::with_capacity(paths.len());
        let mut old_paths: Vec<MPPath> = Vec::with_capacity(paths.len());
        for path in paths.iter() {
            let mut seen = false;
            for opath in old_paths.iter() {
                if path.len() > opath.len() && path.starts_with(opath) {
                    seen = true;
                    break;
                }
            }
            if !seen {
                let (new_path, old_path) = path.substitute_owning(matching.clone());
                if old_path.is_some() {
                    old_paths.push(old_path.unwrap());
                    new_paths.push(new_path);
                } else if new_path.value.is_leaf {
                    new_paths.push(new_path);
                }
            }
        }
        new_paths.shrink_to_fit();
        unsafe { mem::transmute(new_paths) }
    }
}


impl<'a> fmt::Display for MPPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.value)
    }
}

impl<'a> PartialEq for MPPath<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl<'a> Eq for MPPath<'_> {}

impl<'a> Hash for MPPath<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}
