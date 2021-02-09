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


num_rules = 0
num_facts = 0

if __name__ == '__main__':
    args = parser.parse_args()
    t = time.time()
    start = 0
    prolog = Prolog()

    #  prolog.assertz(":- set_prolog_stack(global, limit(1000000000000))")
    #  prolog.assertz(":- set_prolog_stack(trail,  limit(200000000000))")
    #  prolog.assertz(":- set_prolog_stack(local,  limit(20000000000))")

    def print_batch(start):
        global num_rules
        global num_facts
        for n in range(args.i):
            # time.sleep(.5)
            rule_times = []
            fact_times = []
            query_times = []
            tquery_times = []
            query_nums = []
            for x in range(20):
                start += 1
                t_r_1 = time.time()
                prolog.assertz(f"animal{start}(X) :- mammal{start}(X)")
                prolog.assertz(f"mammal{start}(X) :- primate{start}(X)")
                prolog.assertz(f"primate{start}(X) :- human{start}(X)")
                prolog.assertz(f"mortal{start}(X) :- animal{start}(X), living{start}(X)")
                t_r_2 = time.time()
                rule_times.append((t_r_2 - t_r_1) / 4)
                num_rules += 4
                for i in range(args.n):
                    name = f"socrate{start}n{i}"
                    t_f_1 = time.time()
                    prolog.assertz(f"human{start}({name})")
                    prolog.assertz(f"living{start}({name})")
                    t_f_2 = time.time()
                    fact_times.append((t_f_2 - t_f_1) / 2)
                    num_facts += 2
                    num_results = 0
                    t_1 = time.time()
                    for sol in prolog.query(f"mortal{start}(X)"):
                        assert sol['X']
                        num_results += 1
                    query_nums.append(num_results)
                    t_2 = time.time()
                    query_times.append((t_2 - t_1) / num_results)
                    tquery_times.append((t_2 - t_1))

            q_time = (sum(query_times) / len(query_times)) * 1000000
            tq_time = (sum(tquery_times) / len(tquery_times)) * 1000000
            q_nums = sum(query_nums) / len(query_nums)
            r_time = (sum(rule_times) / len(rule_times)) * 1000000
            f_time = (sum(fact_times) / len(fact_times)) * 1000000
            print(f'Rules: {num_rules}, facts: {num_facts}, query time: {tq_time} ({q_time} x {q_nums}), fact time: {f_time}, rule time: {r_time}')

        return start

    for r in range(args.r):
        start += 1
        prolog.assertz(f"animal{start}(X) :- mammal{start}(X)")
        prolog.assertz(f"mammal{start}(X) :- primate{start}(X)")
        prolog.assertz(f"primate{start}(X) :- human{start}(X)")
        prolog.assertz(f"mortal{start}(X) :- animal{start}(X), living{start}(X)")
        num_rules += 4
        for i in range(args.n):
            name = f"socrate{start}n{i}"
            prolog.assertz(f"human{start}({name})")
            prolog.assertz(f"living{start}({name})")
            num_facts += 2

        if r % args.s == 0:
            start = print_batch(start)

    print_batch(start)

    print("total Time: ", (time.time() - t))
