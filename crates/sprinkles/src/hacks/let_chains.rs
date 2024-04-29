#[macro_export]
macro_rules! let_chain {
    (let $dis:ident($pat:ident) = $expr:expr; $(let $dis2:ident($pat2:ident) = $expr2:expr ;)+ $then:expr $(; else $else:expr)?) => {{
        if let $dis($pat) = $expr {
            let_chain!($(let $dis2($pat2) = $expr2 ;)+ $then $(; else $else)?)
        }
        $(else { $else })?
    }};

    (let $dis:ident($pat:ident) = $expr:expr; $then:expr $(; else $else:expr)?) => {{
        if let $dis($pat) = $expr {
            $then
        }
        $(else { $else })?
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_let_chain() {
        let result = let_chain!(let Some(x) = Some(1); let Some(y) = Some(2); let Some(z) = Some(3); {
            (x, y, z)
        }; else panic!("nope"));

        assert_eq!(result, (1, 2, 3));
    }
}
