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
import clips


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

    #  prolog.assertz(":- set_prolog_stack(global, limit(1000000000000))")
    #  prolog.assertz(":- set_prolog_stack(trail,  limit(200000000000))")
    #  prolog.assertz(":- set_prolog_stack(local,  limit(20000000000))")

    def print_batch(start):
        global num_rules
        global num_facts
        for n in range(args.i):
            # time.sleep(.5)
            start += 1
            # t_r_1 = time.time()

            clips.BuildRule("one%s" % start, "(mammal%s ?x1)" % start, "(assert (animal%s ?x1))" % start, "animal%s" % start)
            clips.BuildRule("two%s" % start, "(primate%s ?x1)" % start, "(assert (mammal%s ?x1))" % start, "mammal%s" % start)
            clips.BuildRule("thr%s" % start, "(human%s ?x1)" % start, "(assert (primate%s ?x1))" % start, "primate%s" % start)
            clips.BuildRule("fou%s" % start, "(human%s ?x1) (living%s ?x1)" % (start, start), "(assert (mortal%s ?x1))" % start, "primate%s" % start)

            # t_r_2 = time.time()
            # r_time = ((t_r_2 - t_r_1) / 4) * 1e6
            num_rules += 4
            for i in range(args.n - 1):
                name = "socrate%sn%s" % (start, i)

                clips.Assert("(human%s %s)" % (start, name))
                clips.Assert("(living%s %s)" % (start, name))

                num_facts += 2

                clips.Run()

            name = "socrate%s" % start
            # t_f_1 = time.time()
            clips.Assert("(human%s %s)" % (start, name))
            clips.Assert("(living%s %s)" % (start, name))
            # t_f_2 = time.time()
            # f_time = ((t_f_2 - t_f_1) / 2) * 1e6
            num_facts += 2
            t_1 = time.time()
            clips.Run()
            # query
            t_2 = time.time()
            tq_time = (t_2 - t_1) * 1e6

            # print('Rules: %s, facts: %s, query time: %s , fact time: %s, rule time: %s' % (num_rules, num_facts, tq_time, f_time, r_time))
            print('%s %s' % (args.n, tq_time))

        return start

    for r in range(args.r):
        start += 1
        clips.BuildRule("one%s" % start, "(mammal%s ?x1)" % start, "(assert (animal%s ?x1))" % start, "animal%s" % start)
        clips.BuildRule("two%s" % start, "(primate%s ?x1)" % start, "(assert (mammal%s ?x1))" % start, "mammal%s" % start)
        clips.BuildRule("thr%s" % start, "(human%s ?x1)" % start, "(assert (primate%s ?x1))" % start, "primate%s" % start)
        clips.BuildRule("fou%s" % start, "(human%s ?x1) (living%s ?x1)" % (start, start), "(assert (mortal%s ?x1))" % start, "primate%s" % start)
        num_rules += 4
        for i in range(args.n):
            name = "socrate%sn%s" % (start, i)
            clips.Assert("(human%s %s)" % (start, name))
            clips.Assert("(living%s %s)" % (start, name))
            num_facts += 2

        if r % args.s == 0:
            clips.Run()
            start = print_batch(start)

    print_batch(start)

    # print("total Time: ", (time.time() - t))

