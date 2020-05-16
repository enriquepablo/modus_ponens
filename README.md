# modus_ponens

Modus_ponens is a rust library that can be used to build forward chaining inference engines,
a.k.a. production rule systems. If you need such a system, these are the reasons that might make
modus_ponens interesting to you:

* It is fast.
* It provides total freedom in the syntax of the facts that can be fed to the inference engines
  produced with modus_ponens.
* It is scalable, in the sense that the algorithmic cost (time and space) of adding
  both new facts and new rules to the system is independent of the amount of them already there.
  In this sense it must be noted that it uses a novel algorithm with little resemblance to RETE.
  The results below show that adding a single rule or fact to the system took a few nanoseconds, 
  both into an empty system and into a system with 2e5 rules and 6e5 facts.

These properties make it very appropriate for large and expresive expert systems.

However, it must also be said that

* It is a work in progress. At the moment it doesn't even have arithemetic facilities,
  nor some form of persistance. If I publish this now is because the results below show promise,
  and perhaps they might push someone else into supporting the project.

Below, I will try to substantiate the claims I have made above.

## Inference engines.

Inference engines deal with 2 basic kinds of objects: facts and rules.
The fundamental operational semantics of these objects, in forward chaining systems,
is three-fold:
1) Facts and rules are added by the user to the system;
2) Facts can match with rules, thus producing new facts not directly provided by the user
   (or, equivalently, triggering some other arbitrary actions).
3) The system can be queried for the presence of facts, according to some query language.

Popular examples of forward chaining inference engines are the one in the CLIPS programming language,
or the one behind the Drools Business Rules Management System.
For backward chaining the example would have to be Prolog, or also Drools.

Different engines provide different syntax for their facts.
For example, CLIPS uses lisp style s-expressions,
and Drools uses its own ad-hoc syntax.

Rules are essentially made up of a number of conditions and a number of actions,
conditions are facts that can contain quantified, bound variables,
and actions can be anything to be triggered when the conditions of a rule are matched;
though here for our purposes we will only consider assertions of new facts,
possibly containing variables used in the conditions.

from a logical pow, what these systems provide is, first, a syntax for facts
and for Horn clauses; and then, on top of that, an implementation of conjunction,
implication, and quantified variables, such as they appear in the Horn clauses,
that allows the system to extend any set of facts and Horn clauses to its fullest,
according to modus ponens.

## modus_ponens

What modus_ponens provides is an implementation of logical conjunction and implication and
of quantified variables, and it does so, not on top of some particular syntax for the facts
that are conjoined or implied or that contain the variables, but on top of PEG parse trees.
For modus_ponens, a fact is just a parse tree produced by the Pest PEG parser.
It understands PEG parse nodes, that have a name and text content etc.,
but does not distinguish any of those names or text contents.

Thus, the user of the library can provide whatever PEG she chooses to define her space of facts.
In this PEG, she must mark the productions that can match variables -
and these need not necessarily be terminal productions.
Also, her PEG must include productions accounting for the logical connectives
and for the variables, which are prescribed by modus_ponens;
Otherwise, there is no restriction,
and in particular there is total freedom in the structure of the production providing the facts.

I think that this justifies the claim that modus_ponens provides extreme freedom in choosing
the syntax of the facts to be dealt with.

## example.

As an example, we will develop a system that represents a simple taxonomy,
in which sentences have 2 basic forms, "taxon A is a sub-taxon of taxon B",
and "individual A belongs to taxon B".
We want the system to provide a complete view of our taxonomy;
so for example if we tell the system that Bobby belongs to Dog,
and also that Dog is a sub-taxon of Mammal, and then we query the system for mammals,
we want to obtain Bobby in the response.
For this, we will add 2 rules: "A sub-taxon B & B sub-taxon C -> A sub-taxon C", and
"A belongs B & B sub-taxon C -> A belongs C".

First, the grammar. Since we can use unicode, we'll do so. For "sub-taxon" we'll use ⊆,
and for belongs, ∈. appart from that, we need names for the individuals and taxons,
for which we'll use strings of lower case latin letters,
and variables, with the form Xₙ, where n can be any sequence of digits.

```pest
  WHITESPACE  = { " " | "\t" | "\r" | "\n" }

  knowledge   = { (sentence ~ ".")+ }

  sentence    = _{ rule | fact }

  rule        = { antecedents+ ~ consequents }

  antecedents = { factset ~ "→" }
  consequents = { factset }

  factset     = _{ fact ~ ("∧" ~ fact)* }

  var         = @{ "<__X" ~ ('0'..'9')+ ~ ">" | "X" ~ ("₀".."₉")+ }

  fact        = { name ~ pred ~ name }

  pred        = @{ "∈" | "⊆" }

  v_name      = @{ ASCII_ALPHANUMERIC+ }

  name        = _{ v_name | var }
```

In this grammar, the productions `WHITESPACE`, `knowledge`, `sentence`, `rule`,
`antecedents`, `consequents`, `factset`, and `var` are prescribed by modus_ponens,
although the user has some freedom choosing the form of the variables
(in the present case, the 1st `var` choice, with the form `<__Xn>',
is required by modus_ponens, and the 2nd is not).
On top of these, the user must provide a production for `fact`.
In this case we provide very simple facts, just simple triples subject-predicate-object.

Note how we mark the production `v_name`, that can match variables, with a prefix "v_",
and mix it with `var` in a further `name` production.
We call these *logical* productions. 
In this case `v_name` it is a terminal production, but it need not be so;
and there can be more than one production marked as logical,
so it is perfectly possible to represent higher order logics.

We store this grammar in a file named `grammar.pest`.

now, we build our knowledge base based on the grammar. First some boilerplate:

```rust
extern crate modus_ponens;
#[macro_use]
extern crate modus_ponens_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(KBGen)]
#[grammar = "grammar.pest"]
pub struct KBGenerator;
```

This provides us with a `struct` `KBgenerator`, whose only responsibility is to
create knowledge bases that can hold facts and rules according to `grammar.pest`.
So we can build a knowledge base:

```rust
let kb = KBGenerator::gen_kb();
```

We can add rules to it:

```rust
kb.tell("X₀ ⊆ X₁ ∧ X₁ ⊆ X₂ → X₀ ⊆ X₂.");
kb.tell("X₀ ∈ X₁ ∧ X₁ ⊆ X₂ → X₀ ∈ X₂.");
```

We add some content:

```rust
kb.tell("human ⊆ primate.");
kb.tell("primate ⊆ animal.");
kb.tell("susan ∈ human.");
```

And we query the system:

```rust
assert_eq!(kb.ask("susan ∈ animal.", true);

assert_eq!(kb.ask("susan ⊆ animal.", false);
assert_eq!(kb.ask("primate ∈ animal.", false);
```

That completes a first approach to modus_ponens.

There are some things that are still unsaid,
complex queries, complex logical productions,
consecutive sets of conditions.

## complexity.

We consider here that the state of the art in forward chaining inference engines are implementations
of variants of the RETE algorithm, with different kinds of heuristic improvements
but with no significative change in the fundamental complexity;
and use CLiPS 6.3 as reference implementation of RETE, managed from PyCLIPS.

Now, with modus_ponens, the cost of adding a new fact (or rule) to the system is only dependent
on the grammatical complexity of the fact (or of the conditions of the rule) being added,
and on the number of rules that the fact matches
(or on the number of facts that match a condition of the rule, when adding a rule).
In particular, those costs are independent of both the total number of facts in the system
and the total number of rules in the system.

This is due to the fact that all searches in the structures that represent the sets
of facts and rules in the system are made through hash map lookups;
there is not a single iteration involved.

This is not the case for RETE:
With RETE, the cost of adding a fact or a rule increases with the total number
of rules in the system. At least, that is what the numbers below show. Doorenboss in his thesis
claims that RETE can be adapted to have a match cost independent on the number of rules
under certain conditions (that btw are met by the experiments below),
so I may be mistaken in the sense that CLiPS is state of the art.

The benchmark consisted on adding 200 000 rules and 600 000 facts,
where every 2 rules would be matched by 6 of the facts producing one extra assertion,
and every 1000 rules we would measure the time cost of adding a few more rules and facts.
We am showing the results of 3 runs. Each run took modus_pones around 2 minutes,
and clips around 7 hours.

First we see the effect of increasing the number of rules in the system
on the time the system takes to process each new fact.
CLiPS shows a (seemingly constantly) increasing cost,
whereas modus_ponens persistently takes the same time for each fact.

Some results which are boring and thus we do not show,
gave evidence to the effect that maintining the number of rules,
and increasing the number of facts in the system,
had no effect on the cost of adding new facts or rules,
for any of the systems.
In fact, in the case of modus_ponens the above graph can be taken as evidence that the cost
does not depend on the number of facts,
since for each trial with more rules, the number of facts increased accordingly.

The next results show the effect that increasing the total number of rules
had on the cost of adding a new rule. Again, in clips the cost seems to increase continuously,
whereas in modus_ponens the cost seems independent of the number of rules.

These other results show the memory taken for each fact added to the system,
or rather, the peak memory allocated by the process as measured by heaptrack,
divided y the total number of facts added. As can be seen, the spatial complexity
is independent of the number of both rules and facts as they increase concurrently.

However, there is room for improvement in this sense, as 9kb / fact is way more
than strictly needed.

## algorithm

### data structures

#### facts

We start with a PEG parser, hat has been compiled with a modus_ponens compatible grammar.
Let's suppose a grammar that allows lisp style s-expressions.

If we give a fact to this parser, it will give us back a parse tree.
The nodes that compose the tree have basically 3 properties:

1) a name, that corresponds to the name of the production in the grammar;
2) a text content;
3) information about its position in parent nodes.

For each node, we can think of a "path",
to be the sequence of nodes leading from the root node to the node in question,
including both.

We make 3 distictions among the paths:

* paths can correspond to a variable (variable paths);
* paths can correspond to a node that can match variables (logical paths);
* paths can correspond to a leaf node (leaf paths);

The parse trees are then converted to modus_ponens Facts,
which are basically sequences of paths to certain distinguished nodes.
The distinguished nodes are all the leaf nodes,
(which include the variable nodes),
and the logical nodes.

The nodes in the path are ordered,
with an order given by the order of their text contents within the original textual fact,
and where non terminal nodes go before the terminal nodes whose text they contain.

These paths are hashable and immutable objects.

#### fact tree

The paths that are added to the system are arranged in a tree.
Each node in this tree corresponds to a path in a fact.

Each of these nodes have 2 hash tables of children,
each of which has paths as keys, and nodes as values.
One hash table corresponds to child nodes that are logical,
the other to children that are non-logical.

So to add a fact to the tree, we start by taking its first path.

