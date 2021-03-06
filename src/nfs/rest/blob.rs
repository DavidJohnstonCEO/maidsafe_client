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
use nfs;
use time;
use client;

#[allow(dead_code)]
pub struct Blob {
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>,
    file: nfs::file::File,
}

impl Blob {

    pub fn get_name(&self) -> String {
        self.file.get_metadata().get_name()
    }

    pub fn get_user_metadata(&self) -> Option<Vec<u8>> {
        self.file.get_metadata().get_user_metadata()
    }

    pub fn get_created_time(&self) -> time::Tm {
        self.file.get_metadata().get_created_time()
    }

    pub fn get_modified_time(&self) -> time::Tm {
        self.file.get_metadata().get_modified_time()
    }

    pub fn get_size(&self) -> u64 {
        self.file.get_metadata().get_size()
    }

}

impl nfs::traits::FileWrapper for Blob {

    fn convert_to_file(&self) -> nfs::file::File {
        self.file.clone()
    }

    fn convert_from_file(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>,
        file: nfs::file::File) -> Blob {
        Blob {
            client: client,
            file: file
        }
    }
}
