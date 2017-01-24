// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

#![allow(dead_code)]

extern crate combine;
extern crate edn;
extern crate mentat_query;

use std::collections::BTreeMap;

use self::combine::{any, eof, many, optional, parser, satisfy_map, token, Parser, ParseResult, Stream};
use self::combine::combinator::{Expected, FnParser};
use self::edn::Value::PlainSymbol;
use self::mentat_query::{Element, FindSpec, SrcVar, Variable};

use super::error::{FindParseError, FindParseResult};

pub struct FindSp<I>(::std::marker::PhantomData<fn(I) -> I>);

type FindSpParser<O, I> = Expected<FnParser<I, fn(I) -> ParseResult<O, I>>>;

fn fn_parser<O, I>(f: fn(I) -> ParseResult<O, I>, err: &'static str) -> FindSpParser<O, I>
    where I: Stream<Item = edn::Value>
{
    parser(f).expected(err)
}

impl<I> FindSp<I>
    where I: Stream<Item = edn::Value>
{
   fn variable() -> FindSpParser<Variable, I> {
       fn_parser(FindSp::<I>::variable_, "variable")
   }

   fn variable_(input: I) -> ParseResult<Variable, I> {
       return satisfy_map(|x: edn::Value| super::util::value_to_variable(&x)).parse_stream(input);
   }

   fn period() -> FindSpParser<(), I> {
       fn_parser(FindSp::<I>::period_, "period")
   }

   fn period_(input: I) -> ParseResult<(), I> {
       return satisfy_map(|x: edn::Value| {
           if let PlainSymbol(ref s) = x {
               if s.0.as_str() == "." {
                   return Some(());
               }
           }
           return None;
       }).parse_stream(input);
   }

   fn find_scalar() -> FindSpParser<FindSpec, I> {
       fn_parser(FindSp::<I>::find_scalar_, "find_scalar")
   }

   fn find_scalar_(input: I) -> ParseResult<FindSpec, I> {
       return satisfy_map(|x: edn::Value| if let edn::Value::Vector(y) = x {
           let mut p = (FindSp::variable(), FindSp::period(), eof())
               .map(|(var, _, _)| FindSpec::FindScalar(Element::Variable(var)));
           let r = p.parse_lazy(&y[..]).into();
           match r {
               Ok((r, _)) => Some(r),
               _ => None,
           }
       } else {
           None
       })
       .parse_stream(input);
   }
}
/*
           if let edn::Value::Vector(y) = x {
               let mut p = (FindSp::variable(), eof()).map(|(var, _)| var);
               p.parse_lazy(y.as_slice()).map(|x| x.0)
           } else {
               None
           }
           */

#[test]
fn test_find_sp_variable() {
    let sym = edn::PlainSymbol("?x".to_string());
    let input = [edn::Value::PlainSymbol(sym.clone())];
    let mut parser = FindSp::variable();
    let result = parser.parse(&input[..]);
    assert_eq!(result,
               Ok((Variable(sym), &[][..])));
}

#[test]
fn test_find_scalar() {
    let sym = edn::PlainSymbol("?x".to_string());
    let period = edn::PlainSymbol(".".to_string());
    let input = [edn::Value::PlainSymbol(sym.clone()),
                 edn::Value::PlainSymbol(period.clone())];
    let mut parser = FindSp::find_scalar();
    let result = parser.parse(&input[..]);
    assert_eq!(result,
               Ok((FindSpec::FindScalar(Element::Variable(Variable(sym))), &[][..])));
}

// Parse a sequence of values into one of four find specs.
//
// `:find` must be an array of plain var symbols (?foo), pull expressions, and aggregates.
// For now we only support variables and the annotations necessary to declare which
// flavor of :find we want:
//
//
//     `?x ?y ?z  `     = FindRel
//     `[?x ...]  `     = FindColl
//     `?x .      `     = FindScalar
//     `[?x ?y ?z]`     = FindTuple
//
fn find_seq_to_find_spec(find: &[edn::Value]) -> FindParseResult {
    Err(FindParseError::InvalidInput(find[0].clone()))
}

