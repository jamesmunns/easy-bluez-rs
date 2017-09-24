#![allow(unused_doc_comment)]

use dbus;

error_chain!{
    foreign_links {
        DBus(dbus::Error);
    }
}
