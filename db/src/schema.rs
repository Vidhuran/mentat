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

use entids;
use errors::*;
use types::{Attribute, Entid, EntidMap, IdentMap, Schema, SchemaMap, TypedValue, ValueType};

/// Return `Ok(())` if `schema_map` defines a valid Mentat schema.
fn validate_schema_map(entid_map: &EntidMap, schema_map: &SchemaMap) -> Result<()> {
    for (entid, attribute) in schema_map {
        let ident = entid_map.get(entid).ok_or(ErrorKind::BadSchemaAssertion(format!("Could not get ident for entid: {}", entid)))?;

        if attribute.unique_identity && !attribute.unique_value {
            bail!(ErrorKind::BadSchemaAssertion(format!(":db/unique :db/unique_identity without :db/unique :db/unique_value for entid: {}", ident)))
        }
        if attribute.fulltext && attribute.value_type != ValueType::String {
            bail!(ErrorKind::BadSchemaAssertion(format!(":db/fulltext true without :db/valueType :db.type/string for entid: {}", ident)))
        }
        if attribute.component && attribute.value_type != ValueType::Ref {
            bail!(ErrorKind::BadSchemaAssertion(format!(":db/isComponent true without :db/valueType :db.type/ref for entid: {}", ident)))
        }
        // TODO: consider warning if we have :db/index true for :db/valueType :db.type/string,
        // since this may be inefficient.  More generally, we should try to drive complex
        // :db/valueType (string, uri, json in the future) users to opt-in to some hash-indexing
        // scheme, as discussed in https://github.com/mozilla/mentat/issues/69.
    }
    Ok(())
}

impl Schema {
    pub fn get_ident(&self, x: &Entid) -> Option<&String> {
        self.entid_map.get(x)
    }

    pub fn get_entid(&self, x: &String) -> Option<&Entid> {
        self.ident_map.get(x)
    }

    pub fn attribute_for_entid(&self, x: &Entid) -> Option<&Attribute> {
        self.schema_map.get(x)
    }

    pub fn require_ident(&self, entid: &Entid) -> Result<&String> {
        self.get_ident(&entid).ok_or(ErrorKind::UnrecognizedEntid(*entid).into())
    }

    pub fn require_entid(&self, ident: &String) -> Result<&Entid> {
        self.get_entid(&ident).ok_or(ErrorKind::UnrecognizedIdent(ident.clone()).into())
    }

    pub fn require_attribute_for_entid(&self, entid: &Entid) -> Result<&Attribute> {
        self.attribute_for_entid(entid).ok_or(ErrorKind::UnrecognizedEntid(*entid).into())
    }

    /// Create a valid `Schema` from the constituent maps.
    pub fn from(ident_map: IdentMap, schema_map: SchemaMap) -> Result<Schema> {
        let entid_map: EntidMap = ident_map.iter().map(|(k, v)| (v.clone(), k.clone())).collect();

        validate_schema_map(&entid_map, &schema_map)?;

        Ok(Schema {
            ident_map: ident_map,
            entid_map: entid_map,
            schema_map: schema_map,
        })
    }

    /// Turn vec![(String(:ident), String(:key), TypedValue(:value)), ...] into a Mentat `Schema`.
    pub fn from_ident_map_and_triples<U>(ident_map: IdentMap, assertions: U) -> Result<Schema>
        where U: IntoIterator<Item=(String, String, TypedValue)>{
        let mut schema_map = SchemaMap::new();
        for (ref symbolic_ident, ref symbolic_attr, ref value) in assertions.into_iter() {
            let ident: i64 = *ident_map.get(symbolic_ident).ok_or(ErrorKind::UnrecognizedIdent(symbolic_ident.clone()))?;
            let attr: i64 = *ident_map.get(symbolic_attr).ok_or(ErrorKind::UnrecognizedIdent(symbolic_attr.clone()))?;
            let attributes = schema_map.entry(ident).or_insert(Attribute::default());

            // TODO: improve error messages throughout.
            match attr {
                entids::DB_VALUE_TYPE => {
                    match *value {
                        TypedValue::Ref(entids::DB_TYPE_REF) => { attributes.value_type = ValueType::Ref; },
                        TypedValue::Ref(entids::DB_TYPE_BOOLEAN) => { attributes.value_type = ValueType::Boolean; },
                        TypedValue::Ref(entids::DB_TYPE_LONG) => { attributes.value_type = ValueType::Long; },
                        TypedValue::Ref(entids::DB_TYPE_STRING) => { attributes.value_type = ValueType::String; },
                        TypedValue::Ref(entids::DB_TYPE_KEYWORD) => { attributes.value_type = ValueType::Keyword; },
                        _ => bail!(ErrorKind::BadSchemaAssertion(format!("Expected [... :db/valueType :db.type/*] but got [... :db/valueType {:?}] for ident '{}' and attribute '{}'", value, ident, attr)))
                    }
                },

                entids::DB_CARDINALITY => {
                    match *value {
                        TypedValue::Ref(entids::DB_CARDINALITY_MANY) => { attributes.multival = true; },
                        TypedValue::Ref(entids::DB_CARDINALITY_ONE) => { attributes.multival = false; },
                        _ => bail!(ErrorKind::BadSchemaAssertion(format!("Expected [... :db/cardinality :db.cardinality/many|:db.cardinality/one] but got [... :db/cardinality {:?}]", value)))
                    }
                },

                entids::DB_UNIQUE => {
                    match *value {
                        TypedValue::Ref(entids::DB_UNIQUE_VALUE) => { attributes.unique_value = true; },
                        TypedValue::Ref(entids::DB_UNIQUE_IDENTITY) => {
                            attributes.unique_value = true;
                            attributes.unique_identity = true;
                        },
                        _ => bail!(ErrorKind::BadSchemaAssertion(format!("Expected [... :db/unique :db.unique/value|:db.unique/identity] but got [... :db/unique {:?}]", value)))
                    }
                },

                entids::DB_INDEX => {
                    match *value {
                        TypedValue::Boolean(x) => { attributes.index = x },
                        _ => bail!(ErrorKind::BadSchemaAssertion(format!("Expected [... :db/index true|false] but got [... :db/index {:?}]", value)))
                    }
                },

                entids::DB_FULLTEXT => {
                    match *value {
                        TypedValue::Boolean(x) => {
                            attributes.fulltext = x;
                            if attributes.fulltext {
                                attributes.index = true;
                            }
                        },
                        _ => bail!(ErrorKind::BadSchemaAssertion(format!("Expected [... :db/fulltext true|false] but got [... :db/fulltext {:?}]", value)))
                    }
                },

                entids::DB_IS_COMPONENT => {
                    match *value {
                        TypedValue::Boolean(x) => { attributes.component = x },
                        _ => bail!(ErrorKind::BadSchemaAssertion(format!("Expected [... :db/isComponent true|false] but got [... :db/isComponent {:?}]", value)))
                    }
                },

                entids::DB_DOC => {
                    // Nothing for now.
                },

                entids::DB_IDENT => {
                    // Nothing for now.
                },

                entids::DB_INSTALL_ATTRIBUTE => {
                    // Nothing for now.
                },

                _ => {
                    bail!(ErrorKind::BadSchemaAssertion(format!("Do not recognize attribute '{}' for ident '{}'", attr, ident)))
                }
            }
        };

        Schema::from(ident_map.clone(), schema_map)
    }
}
