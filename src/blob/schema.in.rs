// Copyright 2014 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Table schemas.

table! {
    blobs {
        id -> BigInt,
        name -> Binary,
        tag -> Integer,
    }
}


// Rust models.

#[derive(Queryable)]
pub struct Blob {
    pub id: i64,
    pub name: Vec<u8>,
    pub tag: i32,
}

#[insertable_into(blobs)]
pub struct NewBlob<'a> {
    pub id: i64,
    pub name: &'a [u8],
    pub tag: i32,
}
