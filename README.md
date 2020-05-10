# modus_ponens

Modus_ponens is a rust library that can be used to build forward chaining inference engines,
a.k.a. production rule systems.

These are the claims I make about it:

* It is fast.
* It provides total freedom to the user in choosing the syntax of the facts
  that the inference engines produced with modus_ponens will understand.
* It uses an algorithm that is superior in important ways to the current state of the art.
* It is a work in progress.

Below, I try to substantiate these claims.

## Inference engines.

In essence, inference engines deal with 2 basic kinds of objects: facts and rules.
The facts and rules are in principle provided by the user of the system,
and their operational semantics, in forward chaining systems, is two-fold:
1) The system can be queried for the presence of facts, according to some query language
(or, equivalently, facts can have some additional arbitrary ad-hoc semantics,
such as in a Bussiness rules system);
2) Facts can match with rules, thus producing new facts not directly provided by the user
(or, equivalently, triggering some other arbitrary actions).

Popular examples of inference engines are the CLIPS programming language,
and the inference engine behind the Drools Business Rules Management System.

The syntax of the facts that the system understands depends on the particular system.
Rules are generally made up of a number of conditions,
which are facts that can contain quantified, bound variables,
and a number of actions, that for our purposes we can consider as a set of facts
to be added when all conditions are matched,
possibly containing variables used in the conditions.

from a logical pow, what these systems provide is first a syntax for facts,
and for Horn clauses; and then, on top of that, an implementation of conjunction,
implication, and quantified variables, such as they appear in the Horn clauses,
that automatically extends any set of facts and Horn clauses to its fullest
according to modus ponens.

## modus_ponens

What modus_ponens provides is an implementation of logical conjunction and implication and
of quantified variables, and it does so, not on top of some particular syntax for the facts
that are conjoined or implied or that contain the variables, but on top of PEG parse trees.

From the pow of modus_ponens, a fact is just a parse tree produced by a PEG parser (Pest, in particular).
Modus_ponens understands PEG nodes, that have a name and text content etc.,
but does not distinguish any of those names or text contents,
except for the productions corresponding to our basic logical connectives.

Thus, the user of the library can provide whatever PEG she chooses to define her space of facts.
In this PEG, she must mark the productions that can match variables -
and these need not necessarily be terminal productions :D.
Also, her PEG must include productions for the implication symbol, the conjunction symbol,
a pattern for variables, and a termination symbol, which are prescribed by modus_ponens;
Otherwise, there is no restriction.

I think that this justifies the claim that modus_ponens provides extreme freedom in choosing
the syntax of the facts to be dealt with.

## example.

As an example, I will develop a system that represents a simple taxonomy (think Linnaeus),
in which sentences have 2 basic forms, "taxon A is a sub-taxon of taxon B",
and "individual A belongs to taxon B".
We want to have a complete view of our taxonomy; if we tell the system that Bobby belongs to Dog,
and also that Dog is a sub-taxon of Mammal, and the we query the system for mammals,
we want to obtain Bobby in the response. 
For this, we add 2 rules: "A sub-taxon B & B sub-taxon C -> A sub-taxon C", and
"A belongs B & B sub-taxon C -> A belongs C".

First, the grammar. Since we can use unicode, we'll do so. For "sub-taxon" we'll use C,
and for belongs, E. appart from that, we need names for the individuals and taxons,
for which we'll use sequences of upper case latin letters,
and variables, with the form X sub n, where n can be any sequence of digits.

now, we build our knowledge base based on the grammar:

note how we mark the production that can match variables with a prefix "v_".
In this case it is a terminal production, but it need not be so;
and there can be more than one production marked so,
so it is perfectly possible to represent higher order systems.

We add our 2 rules:

We add some content:

And we query the system:

 

## complexity.

I consider here that the state of the art in inference engines are implementations
of variants of the RETE algorithm, with different kinds of heuristic improvements
but with no significative change in the fundamental complexity.

Now, with modus_ponens, the time cost of adding a new fact to the system is only dependent
on the complexity of the PEG that defines the system
(or, rather, on the complexity of the facts that the PEG allows),
and on the number of rules that the fact matches.
In particular, it is independent of both the number of facts in the system
and the number of rules in the system.

This is due to the fact that all searches in the structures that represent the sets
of facts and rules in the system are made through hash map lookups;
there is not a single iteration involved
(other than through the internal structure of the facts being added, etc.).

This is not the case for RETE:
With RETE, the cost of adding a fact increases with the number of rules in the system.

I can back this up with some numbers.

To get these numbers, I have used CLiPS 6.3 as reference implementation of RETE,
managed from PyCLIPS.

These first results show the effect of increasing the number of rules in the system
on the time the system takes to process each new fact
(for details and links to the code see the appendix).
Starting from 100 rules, and increaing up to 50000,
CLiPS shows a continuously increasing cost,
whereas modus_ponens persistently takes the same time for each fact.

Some results which are boring and thus I do not show,
gave evidence to the effect that maintining the number of rules,
and increasing the number of facts in the system,
had no effect on the cost of adding a new fact,
for any of the systems.
In fact, the above graph can be taken as evidence that the cost in modus_ponens
does not depend on the number of facts,
since for each trial with more rules, the number of facts added increased accordingly.

These other results show the memory taken for each fact added to the system,
or rather, the peak memory allocated by the process as measured by heaptrack,
divided y the total number of facts added. As can be seen, the spatial complexity
is independent of the number of both rules and facts as they increase concurrently.

However, there is room for improvement in this sense, 

I believe these ideas might also be interesting to explore
in 
