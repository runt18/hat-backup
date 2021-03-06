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

use std::sync::Arc;

use backend::{MemoryBackend, StoreBackend};
use errors::HatError;
use hat::HatRc;
use hat::family::Family;
use key;
use util::FileIterator;


pub fn setup_hat<B: StoreBackend>(backend: Arc<B>) -> HatRc<B> {
    let max_blob_size = 4 * 1024 * 1024;
    HatRc::new_for_testing(backend, max_blob_size).unwrap()
}

fn setup_family() -> (Arc<MemoryBackend>, HatRc<MemoryBackend>, Family<MemoryBackend>) {
    let backend = Arc::new(MemoryBackend::new());
    let hat = setup_hat(backend.clone());

    let family = "familyname".to_string();
    let fam = hat.open_family(family).unwrap();

    (backend, hat, fam)
}

pub fn entry(name: Vec<u8>) -> key::Entry {
    key::Entry {
        name: name,
        id: None,
        parent_id: None,
        created: None,
        modified: None,
        accessed: None,
        permissions: None,
        user_id: None,
        group_id: None,
        data_hash: None,
        data_length: None,
    }
}

fn snapshot_files<B: StoreBackend>(family: &Family<B>,
                                   files: Vec<(&str, Vec<u8>)>)
                                   -> Result<(), HatError> {
    for (name, contents) in files {
        try!(family.snapshot_direct(entry(name.bytes().collect()),
                                    false,
                                    Some(FileIterator::from_bytes(contents))));
    }
    Ok(())
}

#[test]
fn snapshot_commit() {
    let (_, mut hat, fam) = setup_family();

    snapshot_files(&fam,
                   vec![("name1", vec![0; 1000000]),
                        ("name2", vec![1; 1000000]),
                        ("name3", vec![2; 1000000])])
        .unwrap();

    fam.flush().unwrap();
    hat.commit(&fam, None).unwrap();
    hat.meta_commit().unwrap();

    let (deleted, live) = hat.gc().unwrap();
    assert_eq!(deleted, 0);
    assert!(live > 0);
}

#[test]
fn snapshot_commit_many_empty_files() {
    let (_, mut hat, fam) = setup_family();

    let names: Vec<String> = (0..3000).map(|i| format!("name-{}", i)).collect();
    snapshot_files(&fam, names.iter().map(|n| (n.as_str(), vec![])).collect()).unwrap();

    fam.flush().unwrap();
    hat.commit(&fam, None).unwrap();
    hat.meta_commit().unwrap();

    let (deleted, live) = hat.gc().unwrap();
    assert_eq!(deleted, 0);
    assert!(live > 0);

    hat.deregister(&fam, 1).unwrap();
    let (deleted, live) = hat.gc().unwrap();
    assert!(deleted > 0);
    assert_eq!(live, 0);
}

#[test]
fn snapshot_commit_many_empty_directories() {
    let (_, mut hat, fam) = setup_family();

    for i in 0..3000 {
        fam.snapshot_direct(entry(format!("name-{}", i).bytes().collect()), true, None)
            .unwrap();
    }

    fam.flush().unwrap();
    hat.commit(&fam, None).unwrap();
    hat.meta_commit().unwrap();

    let (deleted, live) = hat.gc().unwrap();
    assert_eq!(deleted, 0);
    assert!(live > 0);

    hat.deregister(&fam, 1).unwrap();
    let (deleted, live) = hat.gc().unwrap();
    assert!(deleted > 0);
    assert_eq!(live, 0);
}

#[test]
fn snapshot_reuse_index() {
    let (_, mut hat, fam) = setup_family();

    let files = vec![("file1", "block1".bytes().collect()),
                     ("file2", "block2".bytes().collect()),
                     ("file3", "block3".bytes().collect()),
                     ("file4", "block1".bytes().collect()),
                     ("file5", "block2".bytes().collect())];

    // Insert hashes.
    snapshot_files(&fam, files.clone()).unwrap();
    fam.flush().unwrap();

    // Reuse hashes.
    snapshot_files(&fam, files.clone()).unwrap();
    snapshot_files(&fam, files.clone()).unwrap();
    fam.flush().unwrap();

    // No commit, so GC removes all the new hashes.
    let (deleted, live) = hat.gc().unwrap();
    assert!(deleted > 0);
    assert_eq!(live, 0);

    // Update index and reinsert hashes.
    snapshot_files(&fam, files.clone()).unwrap();
    fam.flush().unwrap();

    // Commit.
    hat.commit(&fam, None).unwrap();
    let (deleted, live) = hat.gc().unwrap();
    assert_eq!(deleted, 0);
    assert!(live > 0);

    // Inserting again does not increase number of hashes.
    snapshot_files(&fam, files.clone()).unwrap();
    fam.flush().unwrap();
    let (deleted2, live2) = hat.gc().unwrap();
    assert_eq!(live2, live);
    assert_eq!(deleted2, 0);

    // Cleanup: only 1 snapshot was committed.
    hat.deregister(&fam, 1).unwrap();
    let (deleted, live) = hat.gc().unwrap();
    assert!(deleted > 0);
    assert_eq!(live, 0);
}

#[test]
fn snapshot_gc() {
    let (_, mut hat, fam) = setup_family();

    snapshot_files(&fam,
                   vec![("name1", vec![0; 1000000]),
                        ("name2", vec![1; 1000000]),
                        ("name3", vec![2; 1000000])])
        .unwrap();

    fam.flush().unwrap();

    // No commit so everything is deleted.
    let (deleted, live) = hat.gc().unwrap();
    assert!(deleted > 0);
    assert_eq!(live, 0);
}

#[test]
fn recover() {
    // Prepare a snapshot.
    let (backend, mut hat, fam) = setup_family();

    snapshot_files(&fam,
                   vec![("name1", vec![0; 1000000]),
                        ("name2", vec![1; 1000000]),
                        ("name3", vec![2; 1000000])])
        .unwrap();
    fam.flush().unwrap();
    hat.commit(&fam, None).unwrap();
    hat.meta_commit().unwrap();

    let (deleted, live1) = hat.gc().unwrap();
    assert_eq!(deleted, 0);
    assert!(live1 > 0);

    // Create a new hat to wipe the index states.
    let mut hat2 = setup_hat(backend);

    // Recover index states.
    hat2.recover().unwrap();

    // Check that we now reference all the blobs.
    let (deleted, live2) = hat2.gc().unwrap();
    assert_eq!(deleted, 0);
    assert_eq!(live1, live2);

    // Check that we can delete the snapshot.
    hat2.deregister(&fam, 1).unwrap();

    let (deleted, live3) = hat2.gc().unwrap();
    assert!(deleted > 0);
    assert_eq!(live3, 0);
}
