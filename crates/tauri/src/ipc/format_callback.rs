use serde::Serialize;
use serde_json::value::RawValue;
use serialize_to_javascript::Serialized;

use super::CallbackFn;

/// The information about this is quite limited. On Chrome/Edge and Firefox, [the maximum string size is approximately 1 GB](https://stackoverflow.com/a/34958490).
///
/// [From MDN:](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length#description)
///
/// ECMAScript 2016 (ed. 7) established a maximum length of 2^53 - 1 elements. Previously, no maximum length was specified.
///
/// In Firefox, strings have a maximum length of 2\*\*30 - 2 (~1GB). In versions prior to Firefox 65, the maximum length was 2\*\*28 - 1 (~256MB).
const MAX_JSON_STR_LEN: usize = usize::pow(2, 30) - 2;

/// Minimum size JSON needs to be in order to convert it to JSON.parse with [`format_json`].
// TODO: this number should be benchmarked and checked for optimal range, I set 10 KiB arbitrarily
// we don't want to lose the gained object parsing time to extra allocations preparing it
const MIN_JSON_PARSE_LEN: usize = 10_240;

/// Transforms & escapes a JSON value.
///
/// If it's an object or array, JSON.parse('{json}') is used, with the '{json}' string properly escaped.
/// The return value of this function can be safely used on [`eval`](crate::Window#method.eval) calls.
///
/// Single quotes chosen because double quotes are already used in JSON. With single quotes, we only
/// need to escape strings that include backslashes or single quotes. If we used double quotes, then
/// there would be no cases that a string doesn't need escaping.
///
/// The function takes a closure to handle the escaped string in order to avoid unnecessary allocations.
///
/// # Safety
///
/// The ability to safely escape JSON into a JSON.parse('{json}') relies entirely on 2 things.
///
/// 1. `serde_json`'s ability to correctly escape and format json into a string.
/// 2. JavaScript engines not accepting anything except another unescaped, literal single quote
///     character to end a string that was opened with it.
fn serialize_js_with<T: Serialize, F: FnOnce(&str) -> String>(
    value: &T,
    options: serialize_to_javascript::Options,
    cb: F,
) -> crate::Result<String> {
    // get a raw &str representation of a serialized json value.
    let string = serde_json::to_string(value)?;
    let raw = RawValue::from_string(string)?;

    // from here we know json.len() > 1 because an empty string is not a valid json value.
    let json = raw.get();
    let first = json.as_bytes()[0];

    #[cfg(debug_assertions)]
    if first == b'"' {
        assert!(
            json.len() < MAX_JSON_STR_LEN,
            "passing a string larger than the max JavaScript literal string size"
        )
    }

    let return_val = if json.len() > MIN_JSON_PARSE_LEN && (first == b'{' || first == b'[') {
        let serialized = Serialized::new(&raw, &options).into_string();
        // only use JSON.parse('{arg}') for arrays and objects less than the limit
        // smaller literals do not benefit from being parsed from json
        if serialized.len() < MAX_JSON_STR_LEN {
            cb(&serialized)
        } else {
            cb(json)
        }
    } else {
        cb(json)
    };

    Ok(return_val)
}

/// Formats a function name and argument to be evaluated as callback.
///
/// This will serialize primitive JSON types (e.g. booleans, strings, numbers, etc.) as JavaScript literals,
/// but will serialize arrays and objects whose serialized JSON string is smaller than 1 GB and larger
/// than 10 KiB with `JSON.parse('...')`.
/// See [json-parse-benchmark](https://github.com/GoogleChromeLabs/json-parse-benchmark).
pub fn format<T: Serialize>(function_name: CallbackFn, arg: &T) -> crate::Result<String> {
    serialize_js_with(arg, Default::default(), |arg| {
        format!(
          r#"
    if (window["_{fn}"]) {{
      window["_{fn}"]({arg})
    }} else {{
      console.warn("[TAURI] Couldn't find callback id {fn} in window. This happens when the app is reloaded while Rust is running an asynchronous operation.")
    }}"#,
          fn = function_name.0
        )
    })
}

/// Formats a Result type to its Promise response.
/// Useful for Promises handling.
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
///
/// * `result` the Result to check
/// * `success_callback` the function name of the Ok callback. Usually the `resolve` of the JS Promise.
/// * `error_callback` the function name of the Err callback. Usually the `reject` of the JS Promise.
///
/// Note that the callback strings are automatically generated by the `invoke` helper.
pub fn format_result<T: Serialize, E: Serialize>(
    result: Result<T, E>,
    success_callback: CallbackFn,
    error_callback: CallbackFn,
) -> crate::Result<String> {
    match result {
        Ok(res) => format(success_callback, &res),
        Err(err) => format(error_callback, &err),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for CallbackFn {
        fn arbitrary(g: &mut Gen) -> CallbackFn {
            CallbackFn(u32::arbitrary(g))
        }
    }

    fn serialize_js<T: Serialize>(value: &T) -> crate::Result<String> {
        serialize_js_with(value, Default::default(), |v| v.into())
    }

    #[test]
    fn test_serialize_js() {
        assert_eq!(serialize_js(&()).unwrap(), "null");
        assert_eq!(serialize_js(&5i32).unwrap(), "5");

        #[derive(serde::Serialize)]
        struct JsonObj {
            value: String,
        }

        let raw_str = "T".repeat(MIN_JSON_PARSE_LEN);
        assert_eq!(serialize_js(&raw_str).unwrap(), format!("\"{raw_str}\""));

        assert_eq!(
            serialize_js(&JsonObj {
                value: raw_str.clone()
            })
            .unwrap(),
            format!("JSON.parse('{{\"value\":\"{raw_str}\"}}')")
        );

        assert_eq!(
            serialize_js(&JsonObj {
                value: format!("\"{raw_str}\"")
            })
            .unwrap(),
            format!("JSON.parse('{{\"value\":\"\\\\\"{raw_str}\\\\\"\"}}')")
        );

        let dangerous_json = RawValue::from_string(
      r#"{"test":"don\\🚀🐱‍👤\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don't forget to escape me!","test3":"\\🚀🐱‍👤\\\\'''\\\\🚀🐱‍👤\\\\🚀🐱‍👤\\'''''"}"#.into()
    ).unwrap();

        let definitely_escaped_dangerous_json = format!(
            "JSON.parse('{}')",
            dangerous_json
                .get()
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
        );
        let escape_single_quoted_json_test =
            serialize_to_javascript::Serialized::new(&dangerous_json, &Default::default())
                .into_string();

        let result = r#"JSON.parse('{"test":"don\\\\🚀🐱‍👤\\\\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don\'t forget to escape me!","test3":"\\\\🚀🐱‍👤\\\\\\\\\'\'\'\\\\\\\\🚀🐱‍👤\\\\\\\\🚀🐱‍👤\\\\\'\'\'\'\'"}')"#;
        assert_eq!(definitely_escaped_dangerous_json, result);
        assert_eq!(escape_single_quoted_json_test, result);
    }

    // check arbitrary strings in the format callback function
    #[quickcheck]
    fn qc_formatting(f: CallbackFn, a: String) -> bool {
        // call format callback
        let fc = format(f, &a).unwrap();
        fc.contains(&format!(
            r#"window["_{}"](JSON.parse('{}'))"#,
            f.0,
            serde_json::Value::String(a.clone()),
        )) || fc.contains(&format!(
            r#"window["_{}"]({})"#,
            f.0,
            serde_json::Value::String(a),
        ))
    }

    // check arbitrary strings in format_result
    #[quickcheck]
    fn qc_format_res(result: Result<String, String>, c: CallbackFn, ec: CallbackFn) -> bool {
        let resp = format_result(result.clone(), c, ec).expect("failed to format callback result");
        let (function, value) = match result {
            Ok(v) => (c, v),
            Err(e) => (ec, e),
        };

        resp.contains(&format!(
            r#"window["_{}"]({})"#,
            function.0,
            serde_json::Value::String(value),
        ))
    }
}
