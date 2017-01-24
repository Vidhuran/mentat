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

use self::edn::Value::PlainSymbol;
use self::mentat_query::{FindSpec, SrcVar, Variable};

use super::error::{FindParseError, FindParseResult};

pub fn values_to_variables(vals: &[edn::Value]) -> Result<Vec<Variable>, FindParseError> {
    let mut out: Vec<Variable> = Vec::with_capacity(vals.len());
    for v in vals {
        if let PlainSymbol(ref sym) = *v {
            if sym.0.starts_with('?') {
                out.push(Variable(sym.clone()));
                continue;
            }
        }
        return Err(FindParseError::InvalidInput(v.clone()));
    }
    return Ok(out);
}

#[test]
fn test_values_to_variables() {
    // TODO
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

