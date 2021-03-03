# -*- coding: utf-8 -*-
# Copyright (c) 2019 by Enrique Pérez Arnaud <enrique@cazalla.net>
#
# This file is part of the whatever project.
# https://whatever.cazalla.net
#
# The whatever project is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# The whatever project is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with any part of the terms project.
# If not, see <http://www.gnu.org/licenses/>.
import argparse
import time
from pyswip import Prolog


parser = argparse.ArgumentParser(description='Benchmark on ont.')
parser.add_argument('-d', dest='d', type=int, default=2,
                    help='depth of implications')
parser.add_argument('-n', dest='n', type=int, default=1,
                    help='width of implications')
parser.add_argument('-g', dest='g', type=int, default=0,
                    help='amount of garbage')
parser.add_argument('-r', dest='r', type=int, default=0,
                    help='amount of related garbage')
parser.add_argument('-q', dest='q', type=int, default=0,
                    help='number of queries to make')


num_rules = 0
num_facts = 0

if __name__ == '__main__':
    args = parser.parse_args()
    t_0 = time.time()
    start = 0
    prolog = Prolog()

    animal = "animal"
    living = "living"
    prolog.assertz(f"mortal(X) :- {animal}(X), {living}(X)")

    for d in range(args.d):
        animal_next = f"animal{d}"
        living_next = f"living{d}"
        prolog.assertz(f"{animal}(X) :- {animal_next}(X)")
        prolog.assertz(f"{living}(X) :- {living_next}(X)")
        for g in range(args.g):
            thingy = f"thing{d}n{g}"
            thongy = f"thong{d}n{g}"
            prolog.assertz(f"pre{thingy}(X) :- {thingy}(X)")
            prolog.assertz(f"{thingy}(lattle{thingy})")
            prolog.assertz(f"pre{thongy}(X) :- {thongy}(X)")
            prolog.assertz(f"{thongy}(lattle{thingy})")
        for g in range(args.r):
            thingy = f"thing{d}n{g}"
            thongy = f"thong{d}n{g}"
            prolog.assertz(f"{animal}(X) :- {thingy}(X)")
            prolog.assertz(f"{thingy}(little{thingy})")
            prolog.assertz(f"{living}(X) :- {thongy}(X)")
            prolog.assertz(f"{thongy}(little{thongy})")
        animal = animal_next
        living = living_next

    for n in range(args.n):
        mortal = f"mortal{n}"
        prolog.assertz(f"{animal}({mortal})")
        prolog.assertz(f"{living}({mortal})")

    sols = None
    q_mean = 0.0

    for q in range(args.q):
        t_1 = time.time()
        sols = prolog.query("mortal(X)")
        q_mean += time.time() - t_1

    query_time = (q_mean / args.q) * 1e6

    t_3 = time.time()
    total_time = (t_3 - t_0) * 1e6

    lsols = list(sols)
    num_results = len(lsols)

    print(f"total: {total_time}, query: {query_time}, results: {num_results}")
