#![allow(unused_doc_comment)]

use mvdb::errors as merr;

error_chain!{
    links {
        Mvdb(merr::Error, merr::ErrorKind);
    }
}
