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

use std::collections::HashMap;

use crate::segment::MPSegment;

pub type MPMatching<'a> = HashMap<&'a MPSegment, &'a MPSegment>;

pub fn invert<'a>(matching: &'a MPMatching) -> MPMatching<'a> {
    let mut inverted: MPMatching = HashMap::with_capacity(matching.capacity());
    for (key, value) in matching {
        inverted.insert(value, key);
    }
    inverted
}

pub fn get_or_key<'a>(matching: &'a MPMatching, key: &'a MPSegment) -> &'a MPSegment {
    match matching.get(key) {
        Some(matched) => {
            *matched
            },
        None => {
            key
        }
    }
}


pub fn get_or_key_owning<'a>(matching: MPMatching<'a>, key: &'a MPSegment) -> &'a MPSegment {
    match matching.get(key) {
        Some(matched) => {
            matched
            },
        None => {
            key
        }
    }
}


pub fn get_real_matching<'a>(matching: &MPMatching<'a>, varmap: &MPMatching<'a>) -> MPMatching<'a> {
    let mut real_matching: MPMatching = HashMap::with_capacity(matching.len());
    for (key, value) in matching {
        let new_key = varmap.get(key).unwrap();
        real_matching.insert(&new_key, &value);
    }
    real_matching
}
