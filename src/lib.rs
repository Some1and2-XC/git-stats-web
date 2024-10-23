#![doc(test(attr(warn(unused))))]
#[warn(missing_docs)]
// #[(dead_code)] // I wan't to ignore these issues in the library section of this code base
// #![doc(html_favicon_url = "https://example.com/favicon.ico")]
// #[warn(unused_doc_comment)]

mod library;
pub use library::*;
