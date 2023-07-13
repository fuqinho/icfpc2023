pub fn pretty<U: Into<i64>>(num: U) -> String {
    let num: i64 = num.into();

    if num == 0 {
        return "0".to_string();
    }

    let mut abs = num.abs();

    let mut is = vec![];
    while abs > 0 {
        if abs >= 1000 {
            is.push(format!("{:03}", abs % 1000));
        } else {
            is.push(format!("{}", abs));
        }
        abs /= 1000;
    }
    is.reverse();

    let s = is.join(",");

    if num >= 0 {
        s
    } else {
        format!("-{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::pretty;

    #[test]
    fn test_pretty() {
        assert_eq!(pretty(0i64), "0".to_string());
        assert_eq!(pretty(1i64), "1".to_string());
        assert_eq!(pretty(-1i64), "-1".to_string());
        assert_eq!(pretty(12345i64), "12,345".to_string());
        assert_eq!(pretty(1_010_000_100i64), "1,010,000,100".to_string());
    }
}
