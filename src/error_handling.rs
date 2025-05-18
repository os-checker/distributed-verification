use crate::Result;
pub fn merge_results<I: IntoIterator<Item = (Result<()>, &'static str)>>(results: I) -> Result<()> {
    let mut result = Ok(());
    for (res, msg) in results {
        result = match (res, result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(new), Ok(())) => Err(eyre!("{msg}\n{new:?}\n")),
            (Ok(()), Err(old)) => Err(old),
            (Err(new), Err(old)) => Err(eyre!("{old:?}\n{msg}:\n{new}\n")),
        };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn all_ok() {
        assert!(matches!(merge_results([(Ok(()), ""), (Ok(()), ""), (Ok(()), "")]), Ok(())));
    }

    #[test]
    fn one_error() {
        let res =
            merge_results([(Err(eyre!("ERROR1")), "Msg for error1:"), (Ok(()), ""), (Ok(()), "")]);
        expect![[r#"
            Msg for error1:
            ERROR1

            Location:
                src/error_handling.rs:28:33
        "#]]
        .assert_eq(&res.unwrap_err().to_string());

        let res =
            merge_results([(Ok(()), ""), (Err(eyre!("ERROR1")), "Msg for error1:"), (Ok(()), "")]);
        expect![[r#"
            Msg for error1:
            ERROR1

            Location:
                src/error_handling.rs:39:47
        "#]].assert_eq(&res.unwrap_err().to_string());

        let res =
            merge_results([(Ok(()), ""), (Ok(()), ""), (Err(eyre!("ERROR1")), "Msg for error1:")]);
        expect![[r#"
            Msg for error1:
            ERROR1

            Location:
                src/error_handling.rs:43:61
        "#]].assert_eq(&res.unwrap_err().to_string());
    }

    #[test]
    fn two_errors() {
        let res = merge_results([
            (Ok(()), ""),
            (Err(eyre!("ERROR1")), "Msg for error1:"),
            (Err(eyre!("ERROR2")), "Msg for error2:"),
        ]);
        expect![[r#"
            Msg for error1:
            ERROR1

            Location:
                src/error_handling.rs:51:18


            Location:
                src/error_handling.rs:7:39
            Msg for error2::
            ERROR2
        "#]]
        .assert_eq(&res.unwrap_err().to_string());
    }
}
