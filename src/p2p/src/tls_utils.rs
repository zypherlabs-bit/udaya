use std::io::Write;

/// Generate a self-signed certificate and private key for development/testing
pub fn generate_self_signed_cert(
    cert_path: &str,
    key_path: &str,
    subject: &str,
) -> anyhow::Result<()> {
    let mut params = rcgen::CertificateParams::new(vec![subject.to_string()])?;
    params.distinguished_name = rcgen::DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, subject);
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationName, "Udaya");
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationalUnitName, "Development");
    params
        .distinguished_name
        .push(rcgen::DnType::CountryName, "US");
    params
        .distinguished_name
        .push(rcgen::DnType::LocalityName, "Local");

    let key_pair = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let mut cert_file = std::fs::File::create(cert_path)?;
    cert_file.write_all(cert_pem.as_bytes())?;

    let mut key_file = std::fs::File::create(key_path)?;
    key_file.write_all(key_pem.as_bytes())?;

    Ok(())
}

/// Generate a CA certificate and private key for signing node certificates
pub fn generate_ca_cert(cert_path: &str, key_path: &str) -> anyhow::Result<()> {
    let mut params = rcgen::CertificateParams::new(vec!["Udaya CA".to_string()])?;
    params.distinguished_name = rcgen::DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Udaya CA");
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationName, "Udaya");
    params.distinguished_name.push(
        rcgen::DnType::OrganizationalUnitName,
        "Certificate Authority",
    );
    params
        .distinguished_name
        .push(rcgen::DnType::CountryName, "US");
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    let key_pair = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let mut cert_file = std::fs::File::create(cert_path)?;
    cert_file.write_all(cert_pem.as_bytes())?;

    let mut key_file = std::fs::File::create(key_path)?;
    key_file.write_all(key_pem.as_bytes())?;

    Ok(())
}
