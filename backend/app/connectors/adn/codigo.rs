//! Parsing de códigos de parametrização ADN.

fn normalize_c_trib_nac_6(raw: &str) -> Option<String> {
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

/// Formato ADN `NN.NN.NN.NNN` (desdobro municipal, ex. `000`).
pub fn format_codigo_servico_adn(c_trib_nac: &str, c_trib_mun: &str) -> Option<String> {
    let nac = normalize_c_trib_nac_6(c_trib_nac)?;
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

/// Normaliza desdobro municipal para 3 dígitos.
pub fn normalize_c_trib_mun(raw: &str) -> String {
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

/// `01.01.01.000` → (`010101`, `000`).
pub fn parse_codigo_parametrizacao(codigo: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = codigo
        .split(|c: char| c == '.' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 4 {
        return None;
    }
    let item = parts[0].parse::<u32>().ok()?;
    let sub = parts[1].parse::<u32>().ok()?;
    let des = parts[2].parse::<u32>().ok()?;
    let nac = format!("{:02}{:02}{:02}", item, sub, des);
    let mun = normalize_c_trib_mun(parts[3]);
    Some((nac, mun))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_param_code() {
        let (n, m) = parse_codigo_parametrizacao("01.01.01.001").unwrap();
        assert_eq!(n, "010101");
        assert_eq!(m, "001");
    }
}
