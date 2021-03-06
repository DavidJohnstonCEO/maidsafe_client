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
use client;

pub trait FileWrapper {
    fn convert_to_file(&self) -> nfs::file::File;
    fn convert_from_file(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>,
        file: nfs::file::File) -> Self;
}

pub trait DirectoryListingWrapper {
    fn convert_to_directory_listing(&self) -> nfs::directory_listing::DirectoryListing;
    fn convert_from_directory_listing(client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>,
        directory_listing: nfs::directory_listing::DirectoryListing) -> Self;
}

pub trait DirectoryInfoWrapper {
    fn convert_to_directory_info(&self) -> nfs::directory_info::DirectoryInfo;
    fn convert_from_directory_info(info: nfs::directory_info::DirectoryInfo) -> Self;
}
