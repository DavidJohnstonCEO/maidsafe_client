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

use cbor;
use rand::Rng;
use crypto::buffer::ReadBuffer;
use crypto::buffer::WriteBuffer;

use routing;
use maidsafe_types;
use maidsafe_types::TypeTag;
use routing::sendable::Sendable;

pub mod non_networking_test_framework;

mod user_account;
mod response_getter;
mod callback_interface;

pub struct Client {
    account:             user_account::Account,
    //TODO: Toggle depending on if using actual routing or non_networking_test_framework
    // routing:             ::std::sync::Arc<::std::sync::Mutex<routing::routing_client::RoutingClient<callback_interface::CallbackInterface>>>,
    routing:             ::std::sync::Arc<::std::sync::Mutex<non_networking_test_framework::RoutingClientMock>>,
    response_notifier:   ::ResponseNotifier,
    callback_interface:  ::std::sync::Arc<::std::sync::Mutex<callback_interface::CallbackInterface>>,
    routing_stop_flag:   ::std::sync::Arc<::std::sync::Mutex<bool>>,
    routing_join_handle: Option<::std::thread::JoinHandle<()>>,
}

impl Client {
    //TODO: data_store parameter should be removed when not testing with non_networking_test_framework.
    pub fn create_account(keyword: &String, pin: u32, password: &[u8], data_store: non_networking_test_framework::DataStore) -> Result<Client, ::IoError> {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let account_packet = user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
        let client_id_packet = routing::routing_client::ClientIdPacket::new(account_packet.get_maid().public_keys().clone(),
                                                                            account_packet.get_maid().secret_keys().clone());

        //TODO: Toggle depending on if using actual routing or non_networking_test_framework
        // let routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(routing::routing_client::RoutingClient::new(callback_interface.clone(), client_id_packet)));
        let routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(non_networking_test_framework::RoutingClientMock::new(callback_interface.clone(), data_store)));
        let cloned_routing_client = routing_client.clone();
        let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let routing_stop_flag_clone = routing_stop_flag.clone();

        let client = Client {
            account: account_packet,
            routing: routing_client,
            callback_interface: callback_interface,
            response_notifier: notifier,
            routing_stop_flag: routing_stop_flag,
            routing_join_handle: Some(::std::thread::spawn(move || {
                while !*routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    cloned_routing_client.lock().unwrap().run();
                }
            })),
        };

        {
            let destination = client.account.get_public_maid().name();
            let boxed_public_maid = Box::new(client.account.get_public_maid().clone());
            client.routing.lock().unwrap().unauthorised_put(destination, boxed_public_maid);
        }

        let encrypted_account = maidsafe_types::ImmutableData::new(client.account.encrypt(&password, pin).ok().unwrap());
        let put_res = client.routing.lock().unwrap().put(encrypted_account.clone());
        match put_res {
            Ok(id) => {
                let mut response_getter = response_getter::ResponseGetter::new(id, client.response_notifier.clone(), client.callback_interface.clone());
                match response_getter.get() {
                    Ok(_) => {},
                    Err(_) => return Err(::IoError::new(::std::io::ErrorKind::Other, "Session-Packet PUT-Response Failure !!")),
                }

                let account_version = maidsafe_types::StructuredData::new(user_account::Account::generate_network_id(&keyword, pin),
                                                                          client.account.get_public_maid().name(),
                                                                          vec![encrypted_account.name()]);
                let put_res = client.routing.lock().unwrap().put(account_version);

                match put_res {
                    Ok(id) => {
                        let mut response_getter = response_getter::ResponseGetter::new(id, client.response_notifier.clone(), client.callback_interface.clone());
                        match response_getter.get() {
                            Ok(_) => {},
                            Err(_) => return Err(::IoError::new(::std::io::ErrorKind::Other, "Version-Packet PUT-Response Failure !!")),
                        }

                        Ok(client)
                    },
                    Err(io_error) => Err(io_error),
                }
            },
            Err(io_error) => Err(io_error),
        }
    }

    //TODO: data_store parameter should be removed when not testing with non_networking_test_framework.
    pub fn log_in(keyword: &String, pin: u32, password: &[u8], data_store: non_networking_test_framework::DataStore) -> Result<Client, ::IoError> {
        let notifier = ::std::sync::Arc::new((::std::sync::Mutex::new(0), ::std::sync::Condvar::new()));
        let user_network_id = user_account::Account::generate_network_id(keyword, pin);
        let fake_account_packet = user_account::Account::new(None);
        let callback_interface = ::std::sync::Arc::new(::std::sync::Mutex::new(callback_interface::CallbackInterface::new(notifier.clone())));
        let fake_client_id_packet = routing::routing_client::ClientIdPacket::new(fake_account_packet.get_maid().public_keys().clone(),
                                                                                 fake_account_packet.get_maid().secret_keys().clone());

        //TODO: Toggle depending on if using actual routing or non_networking_test_framework
        // let fake_routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(routing::routing_client::RoutingClient::new(callback_interface.clone(), fake_client_id_packet)));
        let fake_routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(non_networking_test_framework::RoutingClientMock::new(callback_interface.clone(), data_store.clone())));
        let cloned_fake_routing_client = fake_routing_client.clone();
        let fake_routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
        let fake_routing_stop_flag_clone = fake_routing_stop_flag.clone();

        struct RAIIThreadExit {
            routing_stop_flag: ::std::sync::Arc<::std::sync::Mutex<bool>>,
            join_handle: Option<::std::thread::JoinHandle<()>>,
        }

        impl Drop for RAIIThreadExit {
            fn drop(&mut self) {
                *self.routing_stop_flag.lock().unwrap() = true;
                self.join_handle.take().unwrap().join().unwrap();
            }
        }

        let _managed_thread = RAIIThreadExit {
            routing_stop_flag: fake_routing_stop_flag,
            join_handle: Some(::std::thread::spawn(move || {
                while !*fake_routing_stop_flag_clone.lock().unwrap() {
                    ::std::thread::sleep_ms(10);
                    cloned_fake_routing_client.lock().unwrap().run();
                }
            })),
        };

        let structured_data_type_id: maidsafe_types::data::StructuredDataTypeTag = unsafe { ::std::mem::uninitialized() };
        let get_result = fake_routing_client.lock().unwrap().get(structured_data_type_id.type_tag(), user_network_id);

        match get_result {
            Ok(id) => {
                let mut response_getter = response_getter::ResponseGetter::new(id, notifier.clone(), callback_interface.clone());
                match response_getter.get() {
                    Ok(raw_data) => {
                        let mut decoder = cbor::Decoder::from_bytes(raw_data);
                        let account_version: maidsafe_types::StructuredData = decoder.decode().next().unwrap().unwrap();

                        match account_version.value().pop() {
                            Some(latest_version) => {
                                let immutable_data_type_id: maidsafe_types::data::ImmutableDataTypeTag = unsafe { ::std::mem::uninitialized() };
                                let get_result = fake_routing_client.lock().unwrap().get(immutable_data_type_id.type_tag(), latest_version);
                                match get_result {
                                    Ok(id) => {
                                        let mut response_getter = response_getter::ResponseGetter::new(id, notifier.clone(), callback_interface.clone());
                                        match response_getter.get() {
                                            Ok(raw_data) => {
                                                let mut decoder = cbor::Decoder::from_bytes(raw_data);
                                                let encrypted_account_packet: maidsafe_types::ImmutableData = decoder.decode().next().unwrap().unwrap();

                                                let decryption_result = user_account::Account::decrypt(&encrypted_account_packet.value()[..], &password, pin);
                                                if decryption_result.is_err() {
                                                    return Err(::IoError::new(::std::io::ErrorKind::Other, "Could Not Decrypt Session Packet !! (Probably Wrong Password)"));
                                                }
                                                let account_packet = decryption_result.ok().unwrap();

                                                let client_id_packet = routing::routing_client::ClientIdPacket::new(account_packet.get_maid().public_keys().clone(),
                                                                                                                    account_packet.get_maid().secret_keys().clone());

                                                //TODO: Toggle depending on if using actual routing or non_networking_test_framework
                                                // let routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(routing::routing_client::RoutingClient::new(callback_interface.clone(), client_id_packet)));
                                                let routing_client = ::std::sync::Arc::new(::std::sync::Mutex::new(non_networking_test_framework::RoutingClientMock::new(callback_interface.clone(), data_store)));
                                                let cloned_routing_client = routing_client.clone();
                                                let routing_stop_flag = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
                                                let routing_stop_flag_clone = routing_stop_flag.clone();

                                                let client = Client {
                                                    account: account_packet,
                                                    routing: routing_client,
                                                    callback_interface: callback_interface,
                                                    response_notifier: notifier,
                                                    routing_stop_flag: routing_stop_flag,
                                                    routing_join_handle: Some(::std::thread::spawn(move || {
                                                        while !*routing_stop_flag_clone.lock().unwrap() {
                                                            ::std::thread::sleep_ms(10);
                                                            cloned_routing_client.lock().unwrap().run();
                                                        }
                                                    })),
                                                };

                                                Ok(client)
                                            },
                                            Err(_) => Err(::IoError::new(::std::io::ErrorKind::Other, "Session Packet (ImmutableData) GET-Response Failure !!")),
                                        }
                                    },
                                    Err(io_error) => Err(io_error),
                                }
                            },
                            None => Err(::IoError::new(::std::io::ErrorKind::Other, "No Session Packet information in retrieved StructuredData !!")),
                        }
                    },
                    Err(_) => Err(::IoError::new(::std::io::ErrorKind::Other, "StructuredData GET-Response Failure !! (Probably Invalid User-ID)")),
                }
            },
            Err(io_error) => Err(io_error),
        }
    }

    pub fn hybrid_encrypt(&self,
                          data_to_encrypt: &[u8],
                          nonce_opt: Option<::sodiumoxide::crypto::asymmetricbox::Nonce>) -> Result<Vec<u8>, ::crypto::symmetriccipher::SymmetricCipherError> {
        let nonce = match nonce_opt {
            Some(nonce) => nonce,
            None => {
                let digest = ::sodiumoxide::crypto::hash::sha256::hash(&self.account.get_public_maid().name().0);
                let mut nonce = ::sodiumoxide::crypto::asymmetricbox::Nonce([0u8; ::sodiumoxide::crypto::asymmetricbox::NONCEBYTES]);
                let min_length = ::std::cmp::min(::sodiumoxide::crypto::asymmetricbox::NONCEBYTES, digest.0.len());
                for it in digest.0.iter().take(min_length).enumerate() {
                    nonce.0[it.0] = *it.1;
                }
                nonce
            },
        };

        let mut key = [0u8; 32];
        let mut iv  = [0u8; 16];

        let mut rand_generator = ::rand::OsRng::new().ok().unwrap();
        rand_generator.fill_bytes(&mut key);
        rand_generator.fill_bytes(&mut iv);

        let mut combined_key_iv: [u8; 48] = unsafe { ::std::mem::uninitialized() };

        for it in key.iter().enumerate() {
            combined_key_iv[it.0] = *it.1;
        }
        for it in iv.iter().enumerate() {
            combined_key_iv[it.0 + 32] = *it.1;
        }

        let mut encryptor = ::crypto::aes::cbc_encryptor(::crypto::aes::KeySize::KeySize256, &key, &iv, ::crypto::blockmodes::PkcsPadding);

        let mut symm_encryption_result = Vec::<u8>::with_capacity(data_to_encrypt.len());

        let mut read_buffer = ::crypto::buffer::RefReadBuffer::new(data_to_encrypt);
        let mut buffer = [0u8; 4096];
        let mut write_buffer = ::crypto::buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));
            symm_encryption_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

            match result {
                ::crypto::buffer::BufferResult::BufferUnderflow => break,
                ::crypto::buffer::BufferResult::BufferOverflow  => {},
            }
        }

        let asymm_encryption_result = ::sodiumoxide::crypto::asymmetricbox::seal(&combined_key_iv,
                                                                                 &nonce,
                                                                                 &self.account.get_public_maid().public_keys().1,
                                                                                 &self.account.get_maid().secret_keys().1);

        let mut encoder = ::cbor::Encoder::from_memory();
        encoder.encode(&[(asymm_encryption_result, symm_encryption_result)]).unwrap();

        Ok(encoder.into_bytes())
    }

    pub fn hybrid_decrypt(&self,
                          data_to_decrypt: &[u8],
                          nonce_opt: Option<::sodiumoxide::crypto::asymmetricbox::Nonce>) -> Option<Vec<u8>> {
        let mut decoder = ::cbor::Decoder::from_bytes(data_to_decrypt);
        let (asymm_encryption_result, symm_encryption_result): (Vec<u8>, Vec<u8>) = decoder.decode().next().unwrap().unwrap();

        let nonce = match nonce_opt {
            Some(nonce) => nonce,
            None => {
                let digest = ::sodiumoxide::crypto::hash::sha256::hash(&self.account.get_public_maid().name().0);
                let mut nonce = ::sodiumoxide::crypto::asymmetricbox::Nonce([0u8; ::sodiumoxide::crypto::asymmetricbox::NONCEBYTES]);
                let min_length = ::std::cmp::min(::sodiumoxide::crypto::asymmetricbox::NONCEBYTES, digest.0.len());
                for it in digest.0.iter().take(min_length).enumerate() {
                    nonce.0[it.0] = *it.1;
                }
                nonce
            },
        };

        match ::sodiumoxide::crypto::asymmetricbox::open(&asymm_encryption_result[..],
                                                         &nonce,
                                                         &self.account.get_public_maid().public_keys().1,
                                                         &self.account.get_maid().secret_keys().1) {
            Some(asymm_decryption_result) => {
                if asymm_decryption_result.len() == 48 {
                    let mut key: [u8; 32] = unsafe { ::std::mem::uninitialized() };
                    let mut iv : [u8; 16] = unsafe { ::std::mem::uninitialized() };

                    for it in asymm_decryption_result.iter().take(32).enumerate() {
                        key[it.0] = *it.1;
                    }
                    for it in asymm_decryption_result.iter().skip(32).enumerate() {
                        iv[it.0] = *it.1;
                    }

                    let mut decryptor = ::crypto::aes::cbc_decryptor(::crypto::aes::KeySize::KeySize256, &key, &iv, ::crypto::blockmodes::PkcsPadding);

                    let mut symm_decryption_result = Vec::<u8>::with_capacity(symm_encryption_result.len());
                    let mut read_buffer = ::crypto::buffer::RefReadBuffer::new(&symm_encryption_result[..]);
                    let mut buffer = [0u8; 4096];
                    let mut write_buffer = ::crypto::buffer::RefWriteBuffer::new(&mut buffer);

                    loop {
                        match decryptor.decrypt(&mut read_buffer, &mut write_buffer, true) {
                            Ok(result) => {
                                symm_decryption_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
                                match result {
                                    ::crypto::buffer::BufferResult::BufferUnderflow => break,
                                    ::crypto::buffer::BufferResult::BufferOverflow  => {},
                                }
                            },
                            Err(_) => return None,
                        }
                    }

                    Some(symm_decryption_result)
                } else {
                    None
                }
            },
            None => None,
        }
    }

    pub fn get_owner(&self) -> routing::NameType {
        self.account.get_public_maid().name()
    }

    pub fn put<T>(&mut self, sendable: T) -> Result<response_getter::ResponseGetter, ::IoError> where T: Sendable {
        match self.routing.lock().unwrap().put(sendable) {
            Ok(id)      => Ok(response_getter::ResponseGetter::new(id, self.response_notifier.clone(), self.callback_interface.clone())),
            Err(io_err) => Err(io_err),
        }
    }

    pub fn get(&mut self, tag_id: u64, name: routing::NameType) -> Result<response_getter::ResponseGetter, ::IoError> {
        match self.routing.lock().unwrap().get(tag_id, name) {
            Ok(id)      => Ok(response_getter::ResponseGetter::new(id, self.response_notifier.clone(), self.callback_interface.clone())),
            Err(io_err) => Err(io_err),
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        *self.routing_stop_flag.lock().unwrap() = true;
        self.routing_join_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn account_creation() {
        let keyword = "Spandan".to_string();
        let password = "Sharma".as_bytes();
        let pin = 1234u32;
        let data_store = ::std::sync::Arc::new(::std::sync::Mutex::new(::std::collections::BTreeMap::new()));
        let result = Client::create_account(&keyword, pin, &password, data_store);
        assert!(result.is_ok());
    }

    #[test]
    fn account_login() {
        let keyword = "Spandan".to_string();
        let password = "Sharma".as_bytes();
        let pin = 1234u32;
        let data_store = ::client::non_networking_test_framework::get_new_data_store();

        // Without Creation Login Should Fail
        let mut result = Client::log_in(&keyword, pin, &password, data_store.clone());
        assert!(result.is_err());

        // Creation should pass
        result = Client::create_account(&keyword, pin, &password, data_store.clone());
        assert!(result.is_ok());

        // Wrong Credentials (Password) - Login should Fail
        let wrong_password = "sharma".as_bytes();
        result = Client::log_in(&keyword, pin, &wrong_password, data_store.clone());
        assert!(result.is_err());

        // Wrong Credentials (Keyword) - Login should Fail
        let wrong_keyword = "spandan".to_string();
        result = Client::log_in(&wrong_keyword, pin, &password, data_store.clone());
        assert!(result.is_err());

        // Wrong Credentials (Pin) - Login should Fail
        let wrong_pin = 1233;
        result = Client::log_in(&keyword, wrong_pin, &password, data_store.clone());
        assert!(result.is_err());

        // Correct Credentials - Login Should Pass
        result = Client::log_in(&keyword, pin, &password, data_store);
        assert!(result.is_ok());
    }

    #[test]
    fn hybrid_encryption_decryption() {
        // Construct Client
        let keyword = "Spandan".to_string();
        let password = "Sharma".as_bytes();
        let pin = 1234u32;
        let data_store = ::std::sync::Arc::new(::std::sync::Mutex::new(::std::collections::BTreeMap::new()));

        let result = Client::create_account(&keyword, pin, &password, data_store);
        assert!(result.is_ok());
        let client = result.ok().unwrap();

        // Identical Plain Texts
        let plain_text_0 = vec![123u8; 1000];
        let plain_text_1 = plain_text_0.clone();

        // Encrypt passing Nonce
        let nonce = ::sodiumoxide::crypto::asymmetricbox::gen_nonce();
        let hybrid_encrypt_0 = client.hybrid_encrypt(&plain_text_0[..], Some(nonce));
        let hybrid_encrypt_1 = client.hybrid_encrypt(&plain_text_1[..], Some(nonce));

        // Encrypt without passing Nonce
        let hybrid_encrypt_2 = client.hybrid_encrypt(&plain_text_0[..], None);
        let hybrid_encrypt_3 = client.hybrid_encrypt(&plain_text_1[..], None);

        assert!(hybrid_encrypt_0.is_ok());
        assert!(hybrid_encrypt_1.is_ok());
        assert!(hybrid_encrypt_2.is_ok());
        assert!(hybrid_encrypt_3.is_ok());

        // Same Plain Texts
        assert_eq!(plain_text_0, plain_text_1);

        // Different Results because of random "iv"
        assert!(hybrid_encrypt_0.clone().ok().unwrap() != hybrid_encrypt_1.clone().ok().unwrap());
        assert!(hybrid_encrypt_0.clone().ok().unwrap() != hybrid_encrypt_2.clone().ok().unwrap());
        assert!(hybrid_encrypt_0.clone().ok().unwrap() != hybrid_encrypt_3.clone().ok().unwrap());
        assert!(hybrid_encrypt_2.clone().ok().unwrap() != hybrid_encrypt_1.clone().ok().unwrap());
        assert!(hybrid_encrypt_2.clone().ok().unwrap() != hybrid_encrypt_3.clone().ok().unwrap());

        // Decrypt with Nonce
        let hybrid_decrypt_0 = client.hybrid_decrypt(&hybrid_encrypt_0.clone().ok().unwrap()[..], Some(nonce));
        let hybrid_decrypt_1 = client.hybrid_decrypt(&hybrid_encrypt_1.ok().unwrap()[..], Some(nonce));

        // Decrypt without Nonce
        let hybrid_decrypt_2 = client.hybrid_decrypt(&hybrid_encrypt_2.ok().unwrap()[..], None);
        let hybrid_decrypt_3 = client.hybrid_decrypt(&hybrid_encrypt_3.clone().ok().unwrap()[..], None);

        // Decryption without passing Nonce for something encrypted with passing Nonce - Should Fail
        let hybrid_decrypt_4 = client.hybrid_decrypt(&hybrid_encrypt_0.ok().unwrap()[..], None);
        // Decryption passing Nonce for something encrypted without passing Nonce - Should Fail
        let hybrid_decrypt_5 = client.hybrid_decrypt(&hybrid_encrypt_3.ok().unwrap()[..], Some(nonce));

        assert!(hybrid_decrypt_0.is_some());
        assert!(hybrid_decrypt_1.is_some());
        assert!(hybrid_decrypt_2.is_some());
        assert!(hybrid_decrypt_3.is_some());

        // Should fail
        assert!(hybrid_decrypt_4.is_none());
        assert!(hybrid_decrypt_5.is_none());

        // Should have decrypted to the same Plain Texts
        assert_eq!(plain_text_0, hybrid_decrypt_0.unwrap());
        assert_eq!(plain_text_1, hybrid_decrypt_1.unwrap());
        assert_eq!(plain_text_0, hybrid_decrypt_2.unwrap());
        assert_eq!(plain_text_1, hybrid_decrypt_3.unwrap());
    }
}
