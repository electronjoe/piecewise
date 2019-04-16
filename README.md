# piecewise

A Rust crate with supporting representation of Piecewise constant functions (values over Intervals) and common operations.

## Summary

Addition of a new Crate named piecewise which supports rigorous piecewise constant function representation and common piecewise function operations, all while operating over generic bound data types provided required traits are met (e.g. can use [units of measure](https://crates.io/crates/uom) in defining piecewise segments).

## Motivation

In working to write a simulation tool with support for e.g. piecewise function representations that enforce units of measure - I was unable to find a suitable candidate.

### Requirements

The Requirements for the library are then:

1. Support for Piecewise segments defined over [general intervals](https://github.com/electronjoe/intervals-general) with bound data types provied via generic
1. Support for values over Piecewise segments having data types provided via generic
1. Support for SmallPiecewise functions, with focus on performance for low-segment-count representations
1. Support for common operations on piecewise functions, e.g. multipliciation, convolution, value lookup, etc

Additional desires:

1. no_std support
1. No use of of panic, assert
1. Minimize error handling by design
1. Make the library hard to use incorrectly

### Requirement Reasoning

As a motivating case consider the use of this Intervals library to build a step function library for physics simulations.  Lets omit for now the design decision to use Interval representations to back the step function library (versus say "change point" annotations).  Commonly in such a setting, you may want:

One may want to be capable of expressing signal contents:

* Over a domain that (optionally) stretched to +/- infinity
* With arbitrary bounds on segements (e.g. Closed here, Left-Half-Open there)
* Use of [Units of Measure](https://crates.io/crates/uom) types to detect user error

Why some of these characteristics? If you cannot specify interval ranges out to +/- infinity, then one must determine in advance just how far out a signal may matter - which is truly a function of your processing chain.  It is awkward and error prone to simply say to oneself "the signal is primarily in 40-50 Ghz, I'll add sidelobe behavior out an additional 5 Ghz and should be fine" - only to have a mixer place contents well outside of this defined 35-55 Ghz in an important band down the chain.   Additionally, if one can support operations on only domain-complete piecewise functions, then error handling and signaling to the user is greatly reduced (one need not consider how to handle operations in which one of the inputs is undefined).

As to arbitrary bounds on segments - it is confusing and awkward to have a signal that spans e.g. 40-50 Ghz but have a library that uses Intervals that are exclusively e.g. [LeftHalfOpen](https://proofwiki.org/wiki/Definition:Real_Interval/Half-Open) Intervals.  Because query of value_at(40 Ghz) -> undefined while value_at(50 Ghz) -> defined.  This leaves sharp edges.  Meanwhile one cannot use exclusively closed intervals (which alleivate this confusion) because then one cannot define piecewise functions with abbuting intervals (one would double-define some domain point/s).  Additinally, representations of signals may very well want to omit a DC component (i.e. open bound at zero).  The motivation for general Interval specification capabilities is quite broad and will crop up in many mathematics and physics contexts.

Finally, supporting (but not requiring) Units of Measure type enforcement of our units seems pretty non-contentious.

## Detailed Design

### Terminology Selection

* Propose use of **Segment** as the building blocks of Piecewise
  * Domain of a Segement is an **[Interval](https://github.com/electronjoe/intervals-general)**
  * Value of a Segemnt is a generic **value**

### SmallPiecewise vs Piecewise

For small piecewise functions, it is likely that simple operations over a primarily stack allocated array will be more performant.  Benchmarks for comparison and validation of this claim will be provided.  The crate will also support large Piecewise functions using appropriate heap centric allocation and logarithmic operations.

### Constructing Piecewise

Describe builder

## Open Design Questions

### Complete Domain Coverage

Should there be Piecewise variants that are guaranteed to contain the entire Domain?
E.g. this would ensure that value_at would always return a value, no need for Optional.

### Support for broader Piecewise functions

Can support for a braoder class of Piecewise functions be engeineered? E.g. Piecewise linear, etc?  The impact of this extension seems to lie in how well the value generic of Segment can be extended to arbitrary functions yet provide lookup semantics that do not complexify our  base case of constant use.