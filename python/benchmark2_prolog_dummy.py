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


num_rules = 0
num_facts = 0

if __name__ == '__main__':
    args = parser.parse_args()
    t_0 = time.time()
    start = 0
    pass

    animal = "animal"
    living = "living"
    pass

    for d in range(args.d):
        animal_next = f"animal{d}"
        living_next = f"living{d}"
        pass
        pass
        for g in range(args.g):
            thingy = f"thing{d}n{g}"
            thongy = f"thong{d}n{g}"
            pass
            pass
            pass
            pass
        for g in range(args.r):
            thingy = f"thing{d}n{g}"
            thongy = f"thong{d}n{g}"
            pass
            pass
            pass
            pass
        animal = animal_next
        living = living_next

    for n in range(args.n):
        mortal = f"mortal{n}"
        pass
        pass

    t_1 = time.time()
    num_results = len(list(range(args.n)))
    t_2 = time.time()
    query_time = (t_2 - t_1) * 1e6
    total_time = (t_2 - t_0) * 1e6

    print(f"total: {total_time}, query: {query_time}, results: {num_results}")

