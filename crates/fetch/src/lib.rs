use attohttpc::RequestBuilder;
use rustls_native_certs::load_native_certs;
use rustls::Certificate;

pub fn fetch<U>(url: U) -> RequestBuilder where U: AsRef<str>, {
    let mut request = attohttpc::get(url);

    for cert in load_native_certs().expect("could not load platform certs") {
        request = request.add_root_certificate(Certificate(cert.0));
    }

    return request;
}

pub use attohttpc;