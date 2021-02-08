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


sets = ('animal', 'mammal', 'primate', 'human')

parser = argparse.ArgumentParser(description='Benchmark on ont.')
parser.add_argument('-n', dest='n', type=int, default=2,
                    help='number of sentences to add')
parser.add_argument('-r', dest='r', type=int, default=1,
                    help='batch of rules to run')
parser.add_argument('-i', dest='i', type=int, default=4,
                    help='number of samples per batch')
parser.add_argument('-s', dest='s', type=int, default=1000,
                    help='take a batch of samples every s rules')


if __name__ == '__main__':
    args = parser.parse_args()
    t = time.time()
    start = 0
    lsets = len(sets)
    num_r = 2
    num_f = 4 + args.n
    prolog = Prolog()

    def print_batch(start):
        for n in range(args.i):
            # time.sleep(.5)
            t_rs = []
            t_fs = []
            for x in range(20):
                start += 1
                t_0 = time.time()
                prolog.assertz(f"animal{start}(X) :- mammal{start}(X)")
                prolog.assertz(f"mammal{start}(X) :- primate{start}(X)")
                prolog.assertz(f"primate{start}(X) :- human{start}(X)")
                prolog.assertz(f"mortal{start}(X) :- animal{start}(X), living{start}(X)")
                t_1 = time.time()
                for i in range(args.n):
                    name = f"socrate{start}n{i}"
                    prolog.assertz(f"human{start}({name})")
                    prolog.assertz(f"living{start}({name})")
                    for sol in prolog.query(f"mortal{start}(X)"):
                        assert sol['X']
                t_2 = time.time()
                t_rs.append(t_1 - t_0)
                t_fs.append(t_2 - t_1)

            t_r = sum(t_rs) / len(t_rs)
            t_f = sum(t_fs) / len(t_fs)
            t_r_1 = t_r * 1000 / num_r
            t_f_1 = t_f * 1000 / num_f
            print('%d %f %d %f' % (num_r * start, t_r_1, num_f * start, t_f_1))

        return start

    for r in range(args.r):
        start += 1
        prolog.assertz(f"animal{start}(X) :- mammal{start}(X)")
        prolog.assertz(f"mammal{start}(X) :- primate{start}(X)")
        prolog.assertz(f"primate{start}(X) :- human{start}(X)")
        prolog.assertz(f"mortal{start}(X) :- animal{start}(X), living{start}(X)")
        for i in range(args.n):
            name = f"socrate{start}n{i}"
            prolog.assertz(f"human{start}({name})")
            prolog.assertz(f"living{start}({name})")

        if r % args.s == 0:
            start = print_batch(start)

    print_batch(start)

    print("total Time: ", (time.time() - t))
