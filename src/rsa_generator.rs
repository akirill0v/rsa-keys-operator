use openssl::{
    error::ErrorStack,
    hash::MessageDigest,
    nid::Nid,
    pkey::PKey,
    rsa::Rsa,
    x509::{X509Name, X509},
};

#[derive(Clone)]
pub struct Generator {
    pub name: String,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub certificate: Vec<u8>,
}

impl Generator {
    pub fn new(bits: u32, nid: String) -> Result<Self, ErrorStack> {
        let rsa = Rsa::generate(bits)?;

        let pkey = PKey::from_rsa(rsa)?;
        let mut name = X509Name::builder()?;
        name.append_entry_by_nid(Nid::COMMONNAME, nid.as_str())?;
        let name = name.build();

        let mut builder = X509::builder()?;
        builder.set_version(2)?;
        builder.set_subject_name(&name)?;
        builder.set_issuer_name(&name)?;
        builder.set_pubkey(&pkey)?;
        builder.sign(&pkey, MessageDigest::sha256())?;
        let generator = Self {
            name: nid,
            certificate: builder.build().to_pem()?,
            private_key: pkey.private_key_to_pem_pkcs8()?,
            public_key: pkey.public_key_to_pem()?,
        };

        Ok(generator)
    }
}
