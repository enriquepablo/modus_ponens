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

use crate::ruletree::MPRule;
use crate::matching::MPMatching;

pub struct ParseResult<'a> {
    pub facts: Vec<&'a str>,
    pub rules: Vec<MPRule<'a>>,
}


#[derive(Debug)]
pub enum Activation<'a> {
    MPRule {
        rule: MPRule<'a>,
        query_rules: bool,
    },
    Fact {
        fact: &'a str,
        matched: Option<MPMatching<'a>>,
        query_rules: bool,
    },
    Match {
        rule: MPRule<'a>,
        matched: Option<MPMatching<'a>>,
        query_rules: bool,
    },
}

impl<'a> Activation<'a> {

    pub fn from_fact(fact: &'a str, matched: Option<MPMatching<'a>>, query_rules: bool) -> Activation<'a> {
        Activation::Fact {
            fact,
            matched,
            query_rules,
        }
    }
    pub fn from_rule(rule: MPRule, query_rules: bool) -> Activation {
        Activation::MPRule {
            rule,
            query_rules,
        }
    }
    pub fn from_matching(rule: MPRule<'a>, matched: Option<MPMatching<'a>>, query_rules: bool) -> Activation<'a> {
        Activation::Match {
            rule,
            matched,
            query_rules,
        }
    }
}
