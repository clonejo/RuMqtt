use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::net::Shutdown;
use std::path::Path;
use std::time::Duration;

use openssl::ssl::{self, SslMethod, SSL_VERIFY_NONE, SSL_VERIFY_PEER};
use openssl::x509::X509FileType;

pub type SslStream = ssl::SslStream<TcpStream>;

use error::Result;

#[derive(Debug)]
pub struct SslContext {
    pub inner: Arc<ssl::SslContext>,
}

impl SslContext {
    pub fn new<CA, C, K>(ca: CA, client_pair: Option<(C, K)>, should_verify_ca: bool) -> Result<Self>
        where CA: AsRef<Path>,
              C: AsRef<Path>,
              K: AsRef<Path>
    {
        let mut ctx: ssl::SslContext = try!(ssl::SslContext::new(SslMethod::Tlsv1_2));
        try!(ctx.set_cipher_list("DEFAULT"));
        try!(ctx.set_CA_file(ca.as_ref()));

        if let Some((cert, key)) = client_pair {
            try!(ctx.set_certificate_file(cert, X509FileType::PEM));
            try!(ctx.set_private_key_file(key, X509FileType::PEM));
        }
        if should_verify_ca {
            ctx.set_verify(SSL_VERIFY_PEER);
        } else {
            ctx.set_verify(SSL_VERIFY_NONE);
        }
        Ok(SslContext { inner: Arc::new(ctx) })
    }

    pub fn connect(&self, stream: TcpStream) -> Result<SslStream> {
        let ssl_stream = try!(ssl::SslStream::connect(&*self.inner, stream));
        Ok(ssl_stream)
    }
}

pub enum NetworkStream {
    Tcp(TcpStream),
    Tls(SslStream),
    None,
}

impl NetworkStream {
    // fn get_ref(&self) -> io::Result<&TcpStream> {
    //     match *self {
    //         NetworkStream::Tcp(ref s) => Ok(s),
    //         NetworkStream::Tls(ref s) => Ok(s.get_ref()),
    // NetworkStream::None => Err(io::Error::new(io::ErrorKind::Other, "No
    // stream!")),
    //     }
    // }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref s) => s.shutdown(how),
            NetworkStream::Tls(ref s) => s.get_ref().shutdown(how),
            NetworkStream::None => Err(io::Error::new(io::ErrorKind::Other, "No stream!")),
        }
    }

    pub fn set_read_timeout(&mut self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref s) => s.set_read_timeout(dur),
            NetworkStream::Tls(ref s) => s.get_ref().set_read_timeout(dur),
            NetworkStream::None => Err(io::Error::new(io::ErrorKind::Other, "No stream!")),
        }
    }
}

impl Read for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.read(buf),
            NetworkStream::Tls(ref mut s) => s.read(buf),
            NetworkStream::None => Err(io::Error::new(io::ErrorKind::Other, "No stream!")),
        }
    }
}

impl Write for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.write(buf),
            NetworkStream::Tls(ref mut s) => s.write(buf),
            NetworkStream::None => Err(io::Error::new(io::ErrorKind::Other, "No stream!")),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            NetworkStream::Tcp(ref mut s) => s.flush(),
            NetworkStream::Tls(ref mut s) => s.flush(),
            NetworkStream::None => Err(io::Error::new(io::ErrorKind::Other, "No stream!")),
        }
    }
}
