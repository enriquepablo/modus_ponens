# modus_ponens.

## Introduction.

[Modus_ponens][0] is a [rust][1] library that can be used to build [forward chaining][2] [inference engines][3],
a.k.a. [production rule systems][4]. If you need such a system, these are the reasons that might make
modus_ponens interesting to you:

* It is fast. With hundreds or thousands of rules loaded in the system,
  it is the same order of magnitude fast as the [CLIPS][5] programming language,
  and with tens and hundreds of thousands of rules, it is an increasing number of orders
  of magnitude faster than CLIPS.
* It is customizable. It provides total freedom in the syntax of the facts that
  can be fed to the inference engines produced with modus_ponens.
* It is scalable. The algorithmic cost (time and space) of adding
  both new facts and new rules to the system is independent of the amount of them already there.
  In this sense it must be noted that it uses a novel algorithm with little resemblance to [RETE][6].
  The results below show that adding a single rule or fact to the system took a few tens of nanoseconds, 
  both into an empty system and into a system with 2e5 rules and 6e5 facts.

These properties should make it very appropriate for large and expresive expert systems.

However, it must also be said that

* It is a work in progress. At the moment it doesn't even have arithmetic facilities,
  nor some form of persistance. If I publish this now it's because the results below show promise,
  and perhaps they might convince someone else into supporting the project.

Below, I will try to substantiate the claims I have made above.

## Inference engines.

Inference engines deal with 2 basic kinds of objects: facts and rules.
The fundamental operational semantics of these objects, in forward chaining systems,
is three-fold:
1) Facts and rules are added by the user to the system;
2) Facts can match with rules, thus producing new facts not directly provided by the user
   (or, equivalently, triggering some other arbitrary actions);
3) The system can be queried for the presence of facts, according to some query language.

Popular examples of forward chaining inference engines are the one in CLIPS,
or the one behind the [Drools][7] Business Rules Management System.

Different engines provide different syntax for their facts.
For example, CLIPS uses [lisp style s-expressions][8],
and Drools uses some own ad-hoc syntax.
Rules are essentially made up of a number of conditions and a number of actions,
where conditions are facts that can contain quantified, bound variables,
and actions can be anything to be triggered when the conditions of a rule are matched;
though here for our purposes we will only consider as actions assertions of new facts,
possibly containing variables used in the conditions.

from a logical pow, what these systems provide is, first, a syntax for facts
and for [Horn clauses][9]; and then, on top of that, an implementation of conjunction,
implication, and quantified variables, such as they appear in the Horn clauses.
This allows these systems to extend any set of facts and Horn clauses to its completion,
according to modus ponens.

## modus_ponens

What modus_ponens provides is an implementation of logical conjunction and implication and
of quantified variables, and it does so, not on top of some particular syntax for the facts
that are conjoined or implied or that contain the variables, but on top of [PEG][10] parse trees.
For modus_ponens, a fact is just a parse tree produced by the [Pest][11] PEG parser.
Thus, the user of the library can provide whatever PEG she chooses to define her space of facts.
In a sense, the user of the library provides the grammar for the facts,
and modus_ponens provides the grammar to build rules out of those facts.
So, the provided PEG must include productions accounting for the logical connectives
and for variables, prescribed by modus_ponens.
As a bridge between what modus_ponens prescribes and what the user ad-libs,
the user needs to mark which of the productions that compose her facts
can match the variables prescribed by modus_ponens.
Otherwise, there is no restriction in the structure of the productions providing the facts.

I think that this justifies the claim that
modus_ponens provides extreme freedom in choosing a syntax for the facts to be dealt with.

## example.

As an example, we will develop a system that represents a simple taxonomy.
In this system, sentences have 2 basic forms:

1) taxon A is a sub-taxon of taxon B
2) individual A belongs to taxon B

We want the system to provide a complete view of our taxonomy;
So, for example, if we tell the system that Bobby belongs to Dog,
and also that Dog is a sub-taxon of Mammal, and then we query the system for mammals,
we want to obtain Bobby in the response.
For this, we will add 2 rules:

1) A ia a sub-taxon of B & B is a sub-taxon of C -> A is a sub-taxon of C
2) A belongs to B & B is a sub-taxon of C -> A belongs to C

First of all, we must add a dependency to our `Cargo.toml`:

```toml
[dependencies]
modus_ponens_derive = "0.1.0"
```

Now, the grammar. Since we can use unicode, we'll do so.
For the "sub-taxon" predicate we'll use `⊆`, and for belongs, `∈`.
We also need names for the individuals and taxons,
for which we'll use strings of lower case latin letters.

```pest
knowledge   = { (sentence ~ ".")+ }

sentence    = _{ rule | fact }

rule        = { antecedents+ ~ consequents }

antecedents = { factset ~ "→" }
consequents = { factset }

factset     = _{ fact ~ ("∧" ~ fact)* }

var         = @{ "<" ~ "__"? ~ "X" ~ ('0'..'9')+ ~ ">" }

fact        = { name ~ pred ~ name }

pred        = @{ "∈" | "⊆" }

v_name      = @{ ASCII_ALPHANUMERIC+ }

name        = _{ v_name | var }

WHITESPACE  = { " " | "\t" | "\r" | "\n" }
```

In this grammar, the productions `WHITESPACE`, `knowledge`, `sentence`, `rule`,
`antecedents`, `consequents`, `factset`, and `var` are prescribed by modus_ponens.
On top of these, the user must provide a production for `fact`.
In this case we provide very simple facts, just triples subject-predicate-object.

Note how we mark the production `v_name`, that can match variables, with a prefix "v_",
and mix it with `var` in a further `name` production.
We call these *logical* productions. 
In this case `v_name` is a terminal production, but it need not be so;
and there can be more than one production marked as logical.
So it is perfectly possible to represent higher order logics.

We store this grammar in a file named `grammar.pest`.

Then, we build our knowledge base based on the grammar. First some boilerplate:

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
kb.tell("<x0> ⊆ <X1> ∧ <X1> ⊆ <X2> → <X0> ⊆ <X2>.");
kb.tell("<X0> ∈ <X1> ∧ <X1> ⊆ <X2> → <X0> ∈ <X2>.");
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
To try the code in this example yourself, you can do as follows:

```bash
$ git clone <modus_ponens mirror>
$ cd modus_ponens/examples/readme-example
$ cargo build --release
$ RUST_LOG=trace ./target/release/readme-example
```

`RUST_LOG=trace` will log to stdout all facts and rules added in the system;
`RUST_LOG=info` will only log facts.

TODO: document queries with variables,
TODO: document consecutive sets of conditions.

## complexity.

We consider here that the state of the art in forward chaining inference engines are implementations
of variants of the RETE algorithm, with different kinds of heuristic improvements
but with no significative change in the fundamental complexity.
We use CLIPS 6.30 as reference implementation of RETE, managed from [PyCLIPS][12].
There is CLIPS 6.31 and 6.4beta, but we gather from their changelogs that
those new versions do not carry algorithmic improvements that would alter the results shown below,
and PyCLIPS is very convenient for benchmarking CLIPS - and only knows about 6.30.

Now, with modus_ponens, the cost of adding a new fact (or rule) to the system is only dependent
on the grammatical complexity of the fact (or of the conditions of the rule) being added,
and on the number of rules that the fact matches
(or on the number of facts that match a condition of the rule, when adding a rule).
In particular, those costs are independent of both the total number of facts in the system
and the total number of rules in the system.

This is due to the fact that all searches in the structures that represent the sets
of facts and rules in the system are made through hash table lookups;
there is not a single filtered iteration of nodes involved.

This is not the case for RETE:
With RETE, the cost of adding a fact or a rule increases with the total number
of rules in the system. At least, that is what the numbers below show.
Doorenboss in [his thesis][13] sets as objective for an efficient matching algorithm
one that is polynomial in the number of facts (WMEs) and sublinear in the number of
rules. He claims the objective to be achievable that with his RETE/UL enhancement of RETE.
What I observe with CLIPS is a performance independent of the number of facts
and linear in the number of rules.

The benchmarks shown below consisted on adding 200 000 rules and 600 000 facts,
where every 2 rules would be matched by 6 of the facts to produce 4 extra assertions.
Every 1000 rules added we would measure the time cost of adding a few more rules and facts.
We are showing the results of 3 runs. Each run took modus_ponens around 2 minutes,
and CLIPS around 7 hours. [This is the code for the CLIPS benchmark][14]
and [this for modus_ponens][15].

First we see the effect of increasing the number of rules in the system
on the time the system takes to process each new fact.
CLIPS shows a (seemingly constantly) increasing cost,
whereas modus_ponens persistently takes the same time for each fact.

![Effect of the number of rules in the system on the time cost of adding a new fact in CLIPS](img/clips-fact.svg)

![Effect of the number of rules in the system on the time cost of adding a new fact in modus_ponens](img/mopo-fact.svg)

Some results which we do not plot,
gave evidence to the effect that maintining the number of rules,
and increasing the number of facts in the system,
had no effect on the cost of adding new facts or rules,
for any of the systems.
In fact, in the case of modus_ponens the above graph can be taken as evidence that the cost
does not depend on the number of facts,
since for each trial with more rules, the number of facts increased accordingly.

The next results show the effect that increasing the total number of rules
had on the cost of adding a new rule. Again, in CLIPS the cost seems to increase continuously,
whereas in modus_ponens the cost seems independent of the number of rules.

![Effect of the number of rules in the system on the time cost of adding a new rule in CLIPS](img/clips-rules.svg)

![Effect of the number of rules in the system on the time cost of adding a new rule in modus_ponens](img/mopo-rules.svg)

I also measured the peak memory allocated by the process as measured by heaptrack,
with different numbers of facts and rules. I don't have enough data to plot it,
but preliminary results show a constant spatial cost per fact of around 2kb,
independently of the number of favts and rules already in the system.
There is room for improvement in this sense, as 2kb / fact is way more
than strictly needed.

## References

[0]:http://www.modus-ponens.net/
[1]:https://www.rust-lang.org
[2]:https://en.wikipedia.org/wiki/Forward_chaining
[3]:https://en.wikipedia.org/wiki/Inference_engine
[4]:https://en.wikipedia.org/wiki/Production_system_%28computer_science%29
[5]:http://www.clipsrules.net/
[6]:https://en.wikipedia.org/wiki/Rete_algorithm
[7]:https://www.drools.org
[8]:https://en.wikipedia.org/wiki/S-expression
[9]:https://en.wikipedia.org/wiki/Horn_clause
[10]:https://en.wikipedia.org/wiki/Parsing_expression_grammar
[11]:https://pest.rs
[12]:https://pyclips.sourceforge.net/web/
[13]:http://reports-archive.adm.cs.cmu.edu/anon/1995/CMU-CS-95-113.pdf

---

© Enrique Pérez Arnaud <enrique at cazalla.net> 2020