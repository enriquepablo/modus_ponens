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
use std::fmt;


#[derive(Debug, Clone)]
pub struct MPSegment {
    pub text: String,
    pub name: String,
    pub is_leaf: bool,
    pub is_var: bool,
    pub in_var_range: bool,
    pub is_empty: bool,
}

impl MPSegment {
    pub fn new(name: String, text: String, is_leaf: bool, is_var: bool, in_var_range: bool) -> MPSegment {
        let is_empty = text.trim().is_empty();
        MPSegment {
            name, text,
            is_leaf, is_var,
            in_var_range, is_empty,
        }
    }
}

impl fmt::Display for MPSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.text)
    }
}

impl PartialEq for MPSegment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.text == other.text
    }
}

impl Eq for MPSegment {}

impl Hash for MPSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.text.hash(state);
        self.is_leaf.hash(state);
    }
}
