#![allow(unused_doc_comment)]

use mvdb::errors as merr;
use blurz::errors as berr;

error_chain!{
    links {
        Mvdb(merr::Error, merr::ErrorKind);
        Blurz(berr::Error, berr::ErrorKind);
    }
}
