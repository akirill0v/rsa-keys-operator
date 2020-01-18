pub fn secret_name(service_name: String) -> String {
    format!("{}-rsa-token", service_name)
}
