// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

///! This module defines the interface and implementation for parsing an EDN
///! input into a structured Datalog query.
///!
///! The query types are defined in the `query` crate, because they
///! are shared between the parser (EDN -> query), the translator
///! (query -> SQL), and the executor (query, SQL -> running code).
///!
///! The query input can be in two forms: a 'flat' human-oriented
///! sequence:
///!
///! ```clojure
///! [:find ?y :in $ ?x :where [?x :foaf/knows ?y]]
///! ```
///!
///! or a more programmatically generable map:
///!
///! ```clojure
///! {:find [?y]
///!  :in [$]
///!  :where [[?x :foaf/knows ?y]]}
///! ```
///!
///! We parse by expanding the array format into four parts, treating them as the four
///! parts of the map.


extern crate edn;
extern crate mentat_query;

use std::collections::BTreeMap;

use self::mentat_query::FindSpec;

pub enum FindParseError {
    InvalidInput(edn::Value),
    EdnParseError(edn::parse::ParseError),
}

pub type FindParseResult = Result<FindSpec, FindParseError>;

fn parse_find_parts(ins: Vec<edn::Value>, with: Vec<edn::Value>, find: Vec<edn::Value>) -> FindParseResult {
    Ok(FindSpec::FindRel(vec!()))
}

fn parse_find_map(map: BTreeMap<edn::Value, edn::Value>) -> FindParseResult {
    parse_find_parts(vec!(), vec!(), vec!())
}

fn parse_find_vec(vec: Vec<edn::Value>) -> FindParseResult {
    // We expect a vector of Keyword, val, val, Keyword, ….
    // We'll walk the whole vector. If we find a keyword we don't recognize,
    // we'll bail out.
    parse_find_parts(vec!(), vec!(), vec!())
}

pub fn parse_find(expr: edn::Value) -> FindParseResult {
    match expr {
        edn::Value::Vector(v) => parse_find_vec(v),
        edn::Value::Map(m)    => parse_find_map(m),
        _                     => Err(FindParseError::InvalidInput(expr))
    }
}

/// Take a vector of EDN values, as would be extracted from an
/// `edn::Value::Vector`, and turn it into a map.
///
/// The vector must consist of subsequences of an initial plain
/// keyword, followed by one or more non-plain-keyword values.
///
/// The plain keywords are used as keys into the resulting map.
/// The values are accumulated into vectors.
///
/// Invalid input causes this function to return `None`.
fn vec_to_keyword_map(vec: Vec<edn::Value>) -> Option<BTreeMap<edn::Keyword, Vec<edn::Value>>> {
    let mut m = BTreeMap::new();

    if vec.is_empty() {
        return Some(m);
    }

    if vec.len() == 1 {
        return None;
    }

    // Turn something like
    //
    //   `[:foo 1 2 3 :bar 4 5 6]`
    //
    // into
    //
    //   `Some((:foo, [1 2 3]))`
    fn step(slice: &[edn::Value]) -> Option<(edn::Keyword, Vec<edn::Value>)> {
        // This can't be right -- we can't handle [:foo 1 2 3 :bar].
        if slice.len() < 2 {
            return None;
        }

        // The first item must be a keyword.
        if let edn::Value::Keyword(ref k) = slice[0] {

            // The second can't be: [:foo :bar 1 2 3] is invalid.
            if slice[1].is_keyword() {
                return None;
            }

            // Accumulate items until we reach the next keyword.
            let mut acc = Vec::new();
            for v in &slice[1..] {
                if v.is_keyword() {
                    break;
                }
                acc.push(v.clone());
            }
            return Some((k.clone(), acc));
        }

        None
    }

    let mut bits = vec.as_slice();
    while !bits.is_empty() {
        match step(bits) {
            Some((k, v)) => {
                bits = &bits[(v.len() + 1)..];

                // Duplicate keys aren't allowed.
                if m.contains_key(&k) {
                    return None;
                }
                m.insert(k, v);
            },
            None => return None,
        }
    }
    return Some(m);
}

#[test]
fn test_vec_to_keyword_map() {
    let foo = edn::symbols::Keyword("foo".to_string());
    let bar = edn::symbols::Keyword("bar".to_string());
    let baz = edn::symbols::Keyword("baz".to_string());

    // [:foo 1 2 3 :bar 4]
    let input = vec!(edn::Value::Keyword(foo.clone()),
                     edn::Value::Integer(1),
                     edn::Value::Integer(2),
                     edn::Value::Integer(3),
                     edn::Value::Keyword(bar.clone()),
                     edn::Value::Integer(4));

    let m = vec_to_keyword_map(input).unwrap();

    assert!(m.contains_key(&foo));
    assert!(m.contains_key(&bar));
    assert!(!m.contains_key(&baz));

    let onetwothree = vec!(edn::Value::Integer(1),
                           edn::Value::Integer(2),
                           edn::Value::Integer(3));
    let four = vec!(edn::Value::Integer(4));

    assert_eq!(m.get(&foo).unwrap(), &onetwothree);
    assert_eq!(m.get(&bar).unwrap(), &four);

    // Trailing keywords aren't allowed.
    assert_eq!(None,
               vec_to_keyword_map(vec!(edn::Value::Keyword(foo.clone()))));
    assert_eq!(None,
               vec_to_keyword_map(vec!(edn::Value::Keyword(foo.clone()),
                                       edn::Value::Integer(2),
                                       edn::Value::Keyword(bar.clone()))));

    // Duplicate keywords aren't allowed.
    assert_eq!(None,
               vec_to_keyword_map(vec!(edn::Value::Keyword(foo.clone()),
                                       edn::Value::Integer(2),
                                       edn::Value::Keyword(foo.clone()),
                                       edn::Value::Integer(1))));

    // Starting with anything but a keyword isn't allowed.
    assert_eq!(None,
               vec_to_keyword_map(vec!(edn::Value::Integer(2),
                                       edn::Value::Keyword(foo.clone()),
                                       edn::Value::Integer(1))));

    // Consecutive keywords aren't allowed.
    assert_eq!(None,
               vec_to_keyword_map(vec!(edn::Value::Keyword(foo.clone()),
                                       edn::Value::Keyword(bar.clone()),
                                       edn::Value::Integer(1))));

    // Empty lists return an empty map.
    assert_eq!(BTreeMap::new(), vec_to_keyword_map(vec!()).unwrap());
}
