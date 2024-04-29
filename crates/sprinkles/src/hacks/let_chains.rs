#[macro_export]
macro_rules! let_chain {
    (let Some($pat:ident) = $expr:expr; $(let Some($pat2:ident) = $expr2:expr $(;)?)+ $(; else $else:expr)?) => {{
        if let Some($pat) = $expr {
            let_chain!($(let Some($pat2) = $expr2 ;)+ $(; else $else)?)
        } else {
            $($else)?
        }
    }};

    (let Some($pat:ident) = $expr:expr; $(; else $else:expr)?) => {{
        if let Some($pat) = $expr {
            $pat
        } else {
            $($else)?
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_let_chain() {
        let_chain!(let Some(x) = Some(1); let Some(y) = Some(2); else 0);
    }
}
