//! Conversão entre `cTribNac` / `cTribMun` e códigos de parametrização ADN.

/// Normaliza código nacional para 6 dígitos (`010101`).
pub fn normalize_c_trib_nac(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let mut s = digits;
    if s.len() > 6 {
        s = s[s.len() - 6..].to_string();
    }
    while s.len() < 6 {
        s.insert(0, '0');
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::adn;

    fn normalize_c_trib_mun(raw: &str) -> String {
        let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
        let mut s = if digits.is_empty() {
            "0".to_string()
        } else if digits.len() > 3 {
            digits[digits.len() - 3..].to_string()
        } else {
            digits
        };
        while s.len() < 3 {
            s.insert(0, '0');
        }
        s
    }

    fn c_trib_nac_to_parametrizacao(c_trib_nac: &str, c_trib_mun: &str) -> Option<String> {
        let nac = normalize_c_trib_nac(c_trib_nac)?;
        let item = &nac[0..2];
        let sub = &nac[2..4];
        let des = &nac[4..6];
        let mun = normalize_c_trib_mun(c_trib_mun);
        Some(format!(
            "{:02}.{:02}.{:02}.{}",
            u32::from_str_radix(item, 10).ok()?,
            u32::from_str_radix(sub, 10).ok()?,
            u32::from_str_radix(des, 10).ok()?,
            mun
        ))
    }

    #[test]
    fn normalize_nac_from_spreadsheet_number() {
        assert_eq!(normalize_c_trib_nac("10101").as_deref(), Some("010101"));
    }

    #[test]
    fn roundtrip_parametrizacao() {
        let (n, m) = adn::parse_codigo_parametrizacao("01.01.01.000").unwrap();
        assert_eq!(n, "010101");
        assert_eq!(m, "000");
        assert_eq!(
            c_trib_nac_to_parametrizacao("010101", "000").as_deref(),
            Some("01.01.01.000")
        );
    }
}
