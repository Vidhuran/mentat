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
    MissingField(edn::Keyword),
    EdnParseError(edn::parse::ParseError),
}

pub type FindParseResult = Result<FindSpec, FindParseError>;

fn parse_find_parts(ins: &[edn::Value], with: &[edn::Value], find: &[edn::Value]) -> FindParseResult {
    Ok(FindSpec::FindRel(vec!()))
}

fn parse_find_map(map: BTreeMap<edn::Keyword, Vec<edn::Value>>) -> FindParseResult {
    // Eagerly awaiting `const fn`.
    let kw_find = edn::Keyword::new("find");
    let kw_in = edn::Keyword::new("in");
    let kw_with = edn::Keyword::new("with");

    if let Some(find) = map.get(&kw_find) {
        if let Some(ins) = map.get(&kw_in) {
            return parse_find_parts(ins, map.get(&kw_with).unwrap_or([edn::Value; 0]()), find);
        } else {
            return Err(FindParseError::MissingField(kw_in));
        }
    } else {
        return Err(FindParseError::MissingField(kw_find));
    }
}

fn parse_find_edn_map(map: BTreeMap<edn::Value, edn::Value>) -> FindParseResult {
    // Every key must be a Keyword. Every value must be a Vec.
    let mut m = BTreeMap::new();

    if map.is_empty() {
        return parse_find_map(m);
    }

    for (k, v) in map {
        if let edn::Value::Keyword(kw) = k {
            if let edn::Value::Vector(vec) = v {
                m.insert(kw, vec);
                continue;
            } else {
                return Err(FindParseError::InvalidInput(v));
            }
        } else {
            return Err(FindParseError::InvalidInput(k));
        }
    }

    parse_find_map(m)
}

pub fn parse_find(expr: edn::Value) -> FindParseResult {
    // No `match` because scoping and use of `expr` in error handling is nuts.
    if let edn::Value::Map(m) = expr {
        return parse_find_edn_map(m);
    }
    if let edn::Value::Vector(ref v) = expr {
        if let Some(m) = vec_to_keyword_map(v) {
            return parse_find_map(m);
        }
    }
    return Err(FindParseError::InvalidInput(expr));
}

/// Take a slice of EDN values, as would be extracted from an
/// `edn::Value::Vector`, and turn it into a map.
///
/// The slice must consist of subsequences of an initial plain
/// keyword, followed by one or more non-plain-keyword values.
///
/// The plain keywords are used as keys into the resulting map.
/// The values are accumulated into vectors.
///
/// Invalid input causes this function to return `None`.
fn vec_to_keyword_map(vec: &[edn::Value]) -> Option<BTreeMap<edn::Keyword, Vec<edn::Value>>> {
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

    let mut bits = vec;
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

    let m = vec_to_keyword_map(&input).unwrap();

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
               vec_to_keyword_map(&vec!(edn::Value::Keyword(foo.clone()))));
    assert_eq!(None,
               vec_to_keyword_map(&vec!(edn::Value::Keyword(foo.clone()),
                                       edn::Value::Integer(2),
                                       edn::Value::Keyword(bar.clone()))));

    // Duplicate keywords aren't allowed.
    assert_eq!(None,
               vec_to_keyword_map(&vec!(edn::Value::Keyword(foo.clone()),
                                        edn::Value::Integer(2),
                                        edn::Value::Keyword(foo.clone()),
                                        edn::Value::Integer(1))));

    // Starting with anything but a keyword isn't allowed.
    assert_eq!(None,
               vec_to_keyword_map(&vec!(edn::Value::Integer(2),
                                       edn::Value::Keyword(foo.clone()),
                                       edn::Value::Integer(1))));

    // Consecutive keywords aren't allowed.
    assert_eq!(None,
               vec_to_keyword_map(&vec!(edn::Value::Keyword(foo.clone()),
                                       edn::Value::Keyword(bar.clone()),
                                       edn::Value::Integer(1))));

    // Empty lists return an empty map.
    assert_eq!(BTreeMap::new(), vec_to_keyword_map(&vec!()).unwrap());
}
