// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::file::File;
use super::directory_info::DirectoryInfo;
use nfs::metadata::Metadata;
use routing;
use std::fmt;

#[allow(dead_code)]
#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DirectoryListing {
    info: DirectoryInfo,
    sub_directories: Vec<DirectoryInfo>,
    files: Vec<File>
}

#[allow(dead_code)]
impl DirectoryListing {
    pub fn new(parent_dir_id: routing::NameType, name: String, user_metadata: Vec<u8>) -> DirectoryListing {
        DirectoryListing {
            info: DirectoryInfo::new(parent_dir_id, Metadata::new(name, user_metadata)),
            sub_directories: Vec::new(),
            files: Vec::new()
        }
    }

    pub fn get_info(&self) -> DirectoryInfo {
        self.info.clone()
    }

    pub fn get_mut_metadata(&mut self) -> &mut Metadata {
        self.info.get_mut_metadata()
    }

    pub fn get_metadata(&self) -> &Metadata {
        self.info.get_metadata()
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.info.set_metadata(metadata);
    }

    pub fn get_user_metadata(&self) -> Option<Vec<u8>> {
        self.info.get_metadata().get_user_metadata()
    }

    pub fn set_user_metadata(&mut self, user_metadata: Vec<u8>) {
        self.info.get_mut_metadata().set_user_metadata(user_metadata);
    }

    pub fn get_parent_dir_id(&self) -> routing::NameType {
        self.info.get_parent_dir_id()
    }

    pub fn add_file(&mut self, file: File) {
        self.files.push(file);
    }

    pub fn get_files(&self) -> Vec<File> {
        self.files.clone()
    }

    pub fn set_files(&mut self, files: Vec<File>) {
        self.files = files;
    }

    pub fn get_sub_directories(&self) -> Vec<DirectoryInfo> {
        self.sub_directories.clone()
    }

    pub fn set_sub_directories(&mut self, dirs: Vec<DirectoryInfo>) {
        self.sub_directories = dirs;
    }

    pub fn set_name(&mut self, name: String) {
        self.info.get_mut_metadata().set_name(name);
    }

    pub fn get_id(&self) -> routing::NameType {
        self.info.get_id()
    }

    pub fn get_name(&self) -> String {
        self.info.get_metadata().get_name()
    }
}

impl fmt::Debug for DirectoryListing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, metadata: {}", self.info.get_id(), self.info.get_metadata())
    }
}

impl fmt::Display for DirectoryListing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, metadata: {}", self.info.get_id(), self.info.get_metadata())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use cbor;
    use routing;

    #[test]
    fn serialise() {
        let obj_before = DirectoryListing::new(routing::NameType([1u8; 64]), "Home".to_string(), "{mime:\"application/json\"}".to_string().into_bytes());

        let mut e = cbor::Encoder::from_memory();
        e.encode(&[&obj_before]).unwrap();

        let mut d = cbor::Decoder::from_bytes(e.as_bytes());
        let obj_after: DirectoryListing = d.decode().next().unwrap().unwrap();

        assert_eq!(obj_before, obj_after);
    }
}
