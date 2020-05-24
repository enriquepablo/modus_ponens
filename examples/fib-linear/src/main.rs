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

extern crate modus_ponens;
#[macro_use]
extern crate modus_ponens_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;


use crate::modus_ponens::kbase::KBGen;
use crate::modus_ponens::kbase::KBase;

mod kb;


fn main() {
    env_logger::init();
    let kb = kb::KBGenerator::gen_kb();
    kb.tell("

        fib <Fst> <Val1> ∧
        {={
            <Snd> = <Fst> + 1
        }=} ∧ {?{
            <Snd> <= 400
        }?}
            →
        fib <Snd> <Val2> ∧
        {={
            <Nxt> = <Snd> + 1 ∧
            <NxtVal> = <Val1> + <Val2>
        }=}
            →
        fib <Nxt> <NxtVal> ◊

    ");

    kb.tell("fib 0 1 ◊");
    kb.tell("fib 1 1 ◊");
}
